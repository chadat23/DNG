use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use jpeg;
mod dng_utils;
mod tags;
mod get_value;
mod trial;

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
    fn to_u32(&self) -> u32{
        use DataType::*;
        match self {
            Byte(u) | Ascii(u) | Undefined(u) => *u as u32,
            // Ascii(u) => *u as u32,
            Short(u) => *u as u32,
            Long(u) => *u as u32,
            Sbyte(i) => *i as u32,
            // Undefined(u) => *u as u32,
            Sshort(i) => *i as u32,
            Slong(i) => *i as u32,
            Float(f) => *f as u32,
            Double(f) => *f as u32,
            _ => panic!("This type can't be cast to a u32!")
        }
    }
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
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

struct IFD {
    numb_of_entries: u16,
    entries: HashMap<u16, DirectoryEntry>, // note that the hashmap doesn't maintain the entry order, see TIFF6.0 P15
    next_ifd_offset: usize,
}

struct DirectoryEntry {
    tag: u16,
    data_type: u16,
    count: usize,
    value_or_offset: u32
}

struct DNG {
    image_file_header: ImageFileHeader,
    ifds: HashMap<usize, IFD>
}

impl DNG {
    // NewSubFileType equal to 0 for the main image
    pub fn open(path: PathBuf) -> Self {
        let encoded_image = fs::read(path).expect("Unable to read file");
        Self::from_encoded_vec(encoded_image)
    }

    pub fn from_encoded_vec(encoded_image: Vec<u8>) -> Self {
        let image_file_header = ImageFileHeader::parse_image_header(&encoded_image);
        let ifds = dng_utils::parse_ifds(&encoded_image, &image_file_header);
        Self {
            image_file_header,
            ifds,
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
}
