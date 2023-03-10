use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use image::Image;

use jpeg;
mod dng_utils;
mod tags;
mod get_value;
mod trial;

use tags::Tag;

// See TIFF6.0 P15/16
enum EntryData {
    Single(DataType),
    Multiple(Vec<DataType>),
}

impl EntryData {
    fn to_vec(&self) -> Vec<DataType> {
        use EntryData::*;
        let c = match self {
            Single(v) => {
                Vec::from([*v])
            },
            Multiple(v) => {
                v.to_owned()
            }
        };
        c
    }

    fn to_value(&self) -> DataType {
        use EntryData::*;
        match self {
            Single(v) => {
                *v
            },
            Multiple(_) => {
                panic!("There are too many values to get just one!")
            }
        }
    }
}

// // See TIFF6.0 P15/16
#[derive(Clone, Copy)]
enum DataType {
    Other(u8),
    Byte(u8),
    Ascii(u8),
    Short(u16),
    Long(u32),
    Rational([u32; 2]),
    Sbyte(i8),
    Undefined(u8),
    Sshort(i16),
    Slong(i32),
    Srational([i32; 2]),
    Float(f32),
    Double(f64),
}

impl DataType {
    fn to_u8(&self) -> u8{
        use DataType::*;
        match self {
            Byte(u) | Ascii(u) | Undefined(u) => *u as u8,
            // Short(u) => *u as u16,
            // Long(u) => *u as u16,
            // Sbyte(i) => *i as u16,
            // Sshort(i) => *i as u16,
            // Slong(i) => *i as u16,
            // Float(f) => *f as u16,
            // Double(f) => *f as u16,
            _ => panic!("This type can't be cast to a u32!")
        }
    }

    fn to_u16(&self) -> u16{
        use DataType::*;
        match self {
            Byte(u) | Ascii(u) | Undefined(u) => *u as u16,
            Short(u) => *u as u16,
            Long(u) => *u as u16,
            Sbyte(i) => *i as u16,
            Sshort(i) => *i as u16,
            Slong(i) => *i as u16,
            Float(f) => *f as u16,
            Double(f) => *f as u16,
            _ => panic!("This type can't be cast to a u32!")
        }
    }

    fn to_u32(&self) -> u32{
        use DataType::*;
        match self {
            Byte(u) | Ascii(u) | Undefined(u) => *u as u32,
            Short(u) => *u as u32,
            Long(u) => *u as u32,
            Sbyte(i) => *i as u32,
            Sshort(i) => *i as u32,
            Slong(i) => *i as u32,
            Float(f) => *f as u32,
            Double(f) => *f as u32,
            _ => panic!("This type can't be cast to a u32!")
        }
    }

    fn to_usize(&self) -> usize{
        use DataType::*;
        match self {
            Byte(u) | Ascii(u) | Undefined(u) => *u as usize,
            Short(u) => *u as usize,
            Long(u) => *u as usize,
            Sbyte(i) => *i as usize,
            Sshort(i) => *i as usize,
            Slong(i) => *i as usize,
            Float(f) => *f as usize,
            Double(f) => *f as usize,
            _ => panic!("This type can't be cast to a u32!")
        }
    }

    pub(crate) fn get_bytes_per_value(data_type: u16) -> u16 {
        match data_type {
            1 | 2 | 6 | 7 => 1,
            3 | 8 => 2,
            4 | 9 | 11 => 4,
            5 | 10 | 12 => 8,
            _ => panic!("Received an invalid Directory Entry Type")
        }
    }

    fn get_entry_value(buffer: &Vec<u8>, data_type: u16, offset: usize, endian: &Endian) -> Self {
        use DataType::*;
        match data_type {
            1 => Byte(get_value::byte(buffer, offset)),
            2 => Ascii(get_value::ascii(buffer, offset)),
            3 => Short(get_value::short(buffer, offset, endian)),
            4 => Long(get_value::long(buffer, offset, endian)),
            5 => Rational(get_value::rational(buffer, offset, endian)),
            6 => Sbyte(get_value::sbyte(buffer, offset)),
            7 => Undefined(get_value::undefined(buffer, offset)),
            8 => Sshort(get_value::sshort(buffer, offset, endian)),
            9 => Slong(get_value::slong(buffer, offset, endian)),
            10 => Srational(get_value::rsational(buffer, offset, endian)),
            11 => Float(get_value::float(buffer, offset, endian)),
            12 => Double(get_value::double(buffer, offset, endian)),
            _ => panic!("That datatype doesn't make sense!")
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Endian {
    Big,
    Little,
}

#[derive(Clone, Debug, PartialEq)]
enum WordSize {
    Thirtytwo,
    Sixtyfour
}

// See TIFF6, Section 2, Image File Header
struct ImageFileHeader {
    endian: Endian,
    word_size: WordSize,
    ifd_offset: usize,
}

impl ImageFileHeader {
    fn parse_image_header(encoded_image: &Vec<u8>) -> Self {
        let endian = dng_utils::get_endian(encoded_image);
        Self { 
            endian: endian.clone(),
            word_size: dng_utils::get_word_size(encoded_image, &endian), 
            ifd_offset: get_value::long(encoded_image, 4, &endian) as usize
        }
    }    
}

struct IFDs {
    ifds: HashMap<usize, IFD>,
    thumbnail: Option<usize>,
    raw_image: Option<usize>,
    exif: Option<usize>,
    xmp: Option<usize>,
}

impl IFDs {
    fn parse_ifds(buffer: &Vec<u8>, image_file_header: &ImageFileHeader) -> Self {
        let mut offset = image_file_header.ifd_offset;
        let mut ifd = HashMap::new();
        ifd.insert(image_file_header.ifd_offset, IFD::parse_ifd(buffer, image_file_header.ifd_offset, &image_file_header.endian));
        let mut ifds = Self { 
            ifds: ifd,
            thumbnail: None,
            raw_image: None,
            exif: None,
            xmp: None,
        };
        ifds.insert_subifds(buffer, &image_file_header.endian);

        ifds.thumbnail = ifds.get_thumbnail_offset(buffer, &image_file_header.endian);
    
        ifds
    }

    fn insert_subifds(&mut self, buffer: &Vec<u8>, endian: &Endian) {
        let mut new_ifds = IFDs { ifds: HashMap::new() , thumbnail: None, raw_image: None, exif: None, xmp: None };
        for ifd in self.ifds.values() {
            if let Some(entry) = ifd.entries.get(&(Tag::SubIFDs_330 as u16)) {
                let ifd_offsets = entry.get_entry_values(buffer, endian);
                let mut sub_ifds = HashMap::new();
                for offset in ifd_offsets.to_vec().iter().map(|f| f.to_usize()) {
                    sub_ifds.insert(offset, IFD::parse_ifd(buffer, offset, &endian));
                }
                let mut ifds = Self { 
                    ifds: sub_ifds,
                    thumbnail: None,
                    raw_image: None,
                    exif: None,
                    xmp: None,
                };
                ifds.insert_subifds(buffer, endian);
                new_ifds.extend(ifds);
            }
        }
        self.extend(new_ifds);
    }

    fn extend(&mut self, other: Self) {
        self.ifds.extend(other.ifds);
    }

    // DNG spec 1.6 SubIFD Trees P12
    // TODO: maybe needs works
    fn get_thumbnail_offset(&self, buffer: &Vec<u8>, endian: &Endian) -> Option<usize> {
        let mut off = usize::MAX;
        for (offset, ifd) in &self.ifds {
            if let Some(new_sub_field) = ifd.entries.get(&(Tag::NewSubFileType_254 as u16)) {
                if new_sub_field.get_entry_values(buffer, endian).to_value().to_u32() == 1 && *offset < off {
                    off = *offset;
                }
            }
        }
        if off < usize::MAX { Some(off) } else { None }
    }

    fn get_thumbnail_idf(&self) -> Option<&IFD> {
        match self.thumbnail {
            Some(offset) => self.ifds.get(&offset),
            None => None
        }
    }
}

struct IFD {
    offset: usize,
    numb_of_entries: u16,
    entries: HashMap<u16, DirectoryEntry>, // note that the hashmap doesn't maintain the entry order, see TIFF6.0 P15
    // next_ifd_offset: usize, // this isn't used / allowed in DNG, only in TIFFs
}

impl IFD {
    fn parse_ifd(buffer: &Vec<u8>, offset: usize, endian: &Endian) -> Self {
        let entry_count = get_value::short(buffer, offset, endian) as usize;    
        let mut entries = HashMap::new();
        for i in 0..entry_count {
            let tag = get_value::short(buffer, offset + 2 + i * 12, endian);
            let data_type = get_value::short(buffer, offset + 4 + i * 12, endian);
            let count = get_value::long(buffer, offset + 6 + i * 12, endian) as usize;
            let value_or_offset = get_value::long(buffer, offset + 10 + i * 12, endian);
    
            entries.insert(tag.clone(), DirectoryEntry { tag, data_type: data_type, count, value_or_offset });
        }
        Self {
            offset,
            numb_of_entries: entry_count as u16,
            entries,
            // next_ifd_offset: get_value::long(buffer, offset + 2 + entry_count * 12, endian) as usize,
        }
    }
}

struct DirectoryEntry {
    tag: u16,
    data_type: u16,
    count: usize,
    value_or_offset: u32
}

// TODO: need to know if it's going to be a vec or singleton
impl DirectoryEntry {
    fn get_entry_values(&self, buffer: &Vec<u8>, endian: &Endian) -> EntryData {
        let bytes_per_value = DataType::get_bytes_per_value(self.data_type) as usize;
        let total_used_bytes = bytes_per_value * self.count;
        match self.count {
            count if count > 1 => {
                let mut multiple = Vec::with_capacity(self.count);
                if total_used_bytes <= 4 {
                    for i in 0..self.count {
                        multiple.push(DataType::get_entry_value(&self.value_or_offset.to_be_bytes().to_vec(), self.data_type, i * bytes_per_value, endian));
                    }
                } else {
                    for i in 0..self.count {
                        multiple.push(DataType::get_entry_value(buffer, self.data_type, self.value_or_offset as usize + i * bytes_per_value, endian));
                    }
                }
                EntryData::Multiple(multiple)
            },
            _ => {
                if total_used_bytes <= 4 {
                    let buffer = self.value_or_offset.to_be_bytes()[4-bytes_per_value..4].to_vec();
                    EntryData::Single(DataType::get_entry_value(&buffer, self.data_type, 0, &Endian::Big))
                } else {
                    let buffer = buffer[self.value_or_offset as usize..self.value_or_offset as usize + bytes_per_value].to_vec();
                    EntryData::Single(DataType::get_entry_value(&buffer, self.data_type, 0, endian))
                }
            }
        }    
    }
}

struct DNG {
    encoded_image: Vec<u8>,
    image_file_header: ImageFileHeader,
    ifds: IFDs
}

impl DNG {
    // NewSubFileType equal to 0 for the main image
    pub fn open(path: PathBuf) -> Self {
        let encoded_image = fs::read(path).expect("Unable to read file");
        Self::from_encoded_vec(encoded_image)
    }

    pub fn from_encoded_vec(encoded_image: Vec<u8>) -> Self {
        let image_file_header = ImageFileHeader::parse_image_header(&encoded_image);
        let ifds = IFDs::parse_ifds(&encoded_image, &image_file_header);
        Self {
            encoded_image,
            image_file_header,
            ifds,
        }
    }

    pub fn get_thumbnail(&self) -> Image {
        let thumbnail_ifd = self.ifds.get_thumbnail_idf().unwrap();

        let width = thumbnail_ifd.entries[&(Tag::ImageWidth_256 as u16)].get_entry_values(&self.encoded_image, &self.image_file_header.endian).to_value().to_u32();
        let length = thumbnail_ifd.entries[&(Tag::ImageLength_257 as u16)].get_entry_values(&self.encoded_image, &self.image_file_header.endian).to_value().to_u32();
        
        let bits_per_sample = thumbnail_ifd.entries[&(Tag::BitsPerSample_258 as u16)].get_entry_values(&self.encoded_image, &self.image_file_header.endian).to_vec();
        let compression = thumbnail_ifd.entries[&(Tag::Compression_259 as u16)].get_entry_values(&self.encoded_image, &self.image_file_header.endian).to_value().to_u16();
        let photometric_interpretation = thumbnail_ifd.entries[&(Tag::PhotometricInterpretation_262 as u16)].get_entry_values(&self.encoded_image, &self.image_file_header.endian).to_value().to_u16();
        
        let strip_offsets = thumbnail_ifd.entries[&(Tag::StripOffsets_273 as u16)].get_entry_values(&self.encoded_image, &self.image_file_header.endian).to_value().to_usize();
        let orientation = thumbnail_ifd.entries[&(Tag::Orientation_274 as u16)].get_entry_values(&self.encoded_image, &self.image_file_header.endian).to_value().to_u16();
        let samples_per_pixel = thumbnail_ifd.entries[&(Tag::SamplesPerPixel_277 as u16)].get_entry_values(&self.encoded_image, &self.image_file_header.endian).to_value().to_u16();
        let rows_per_strip = thumbnail_ifd.entries[&(Tag::RowsPerStrip_278 as u16)].get_entry_values(&self.encoded_image, &self.image_file_header.endian).to_value().to_u32();
        
        let strip_byte_count = thumbnail_ifd.entries[&(Tag::StripByteCounts_279 as u16)].get_entry_values(&self.encoded_image, &self.image_file_header.endian).to_value().to_usize();
        
        let planar_configuration = thumbnail_ifd.entries[&(Tag::PlanarConfiguration_284 as u16)].get_entry_values(&self.encoded_image, &self.image_file_header.endian).to_value().to_u32();

        assert_eq!(bits_per_sample.iter().map(|f| f.to_u16()).collect::<Vec<u16>>(), vec![8, 8, 8]);
        assert_eq!(compression, 1);
        assert_eq!(photometric_interpretation, 2);        
        assert_eq!(orientation, 1);        
        assert_eq!(samples_per_pixel, 3);     
        assert_eq!(rows_per_strip, length);   
        assert_eq!(planar_configuration, 1);   

        let image_data = &dng_utils::get_image_chunks(&self.encoded_image, &vec![strip_offsets], &vec![strip_byte_count])[0];

        let mut data = Vec::with_capacity(image_data.len());
        for &i in image_data.iter() {
            data.push(i);
        }

        let mut tags = thumbnail_ifd.entries.keys().collect::<Vec<&u16>>();
        tags.sort();

        Image {
            data,
            width,
            height: length,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    #[test]
    fn open_working() {
        let mut path = env::current_dir().unwrap();
        path.push("tests/common/RAW_CANON_6D.dng");

        let dng = DNG::open(path);

        assert_eq!(dng.image_file_header.endian, Endian::Little);
        assert_eq!(dng.image_file_header.word_size, WordSize::Thirtytwo);
        assert_eq!(dng.image_file_header.ifd_offset, 8);
    }

    #[test]
    fn thumbnail() {

        let mut path = env::current_dir().unwrap();
        path.push("tests/common/RAW_CANON_6D.dng");

        let dng = DNG::open(path);
        let image = dng.get_thumbnail();
    }
}
