use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use jpeg;
mod dng_utils;
mod tags;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[derive(Clone, Debug, PartialEq)]
enum Endedness {
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
    endedness: Endedness,
    word_size: WordSize,
    ifd_offset: usize,
}

impl ImageFileHeader {
    fn parse_image_header(encoded_image: &Vec<u8>) -> Self {
        let endedness = dng_utils::get_endedness(encoded_image);
        Self { 
            endedness: endedness.clone(),
            word_size: dng_utils::get_word_size(encoded_image), 
            ifd_offset: dng_utils::get_u32_at_idx(encoded_image, 4, &endedness) as usize
        }
    }    
}

struct IFD {
    numb_of_entries: u16,
    entries: HashMap<u16, DirectoryEntry>, // note that the hashmap doesn't maintain the entry order, see TIFF6.0 P15
    next_ifd_offset: usize,
}

// See TIFF6.0 P15/16
enum EntryType {
    Byte = 1,
    Ascii = 2,
    Short = 3,
    Long = 4,
    Rational = 5,
    Sbyte = 6,
    Undefined = 7,
    Sshort = 8,
    Slong = 9,
    Srational = 10,
    Float = 11,
    Double = 12,
}

struct DirectoryEntry {
    tag: u16,
    entry_type: u16,
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

        assert_eq!(dng.image_file_header.endedness, Endedness::Little);
        assert_eq!(dng.image_file_header.word_size, WordSize::Thirtytwo);
        assert_eq!(dng.image_file_header.ifd_offset, 8);
    }
}
