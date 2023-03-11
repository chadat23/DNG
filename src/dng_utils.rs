use std::{collections::HashMap, hash::Hash};

use crate::{ Endian, WordSize, ImageFileHeader, IFD, DirectoryEntry, tags::Tag, get_value, DataType, EntryData};

// See TIFF6, Section 2, Image File Header
pub(crate) fn get_endian(buffer: &Vec<u8>,) -> Endian {
    use Endian::*;
    match get_value::short(buffer, 0, &Big) {
        0x4949 => Little,
        0x4D4D => Big,
        _ => panic!("Endian bytes aren't valid!!!")
    }
}

pub(crate) fn get_word_size(buffer: &Vec<u8>, endian: &Endian) -> WordSize {
    use WordSize::*;
    let idx = match endian {
        Endian::Little => 2,
        Endian::Big => 3,
    };
    match get_value::byte(buffer, idx) {
        42 => Thirtytwo,
        43 => Sixtyfour,
        _ => panic!("Endian bytes aren't valid!!!")
    }
}

// pub(crate) fn parse_ifds(buffer: &Vec<u8>, image_file_header: &ImageFileHeader) -> HashMap<usize, IFD> {
//     let mut ifds = HashMap::new();

//     let mut offset = image_file_header.ifd_offset;
//     while offset > 0 {
//         let ifd = parse_ifd(buffer, &image_file_header.ifd_offset, &image_file_header.endian);
//         if let Some(entry) = ifd.entries.get(&(Tag::SubIFD as u16)) {
//             let ifd_offsets = get_entry_values(buffer, entry, &image_file_header.endian).to_vec().iter().map(|f| f.to_u32()).collect::<Vec<u32>>();
//             let sub_ifds = parse_ifd_tree(buffer, ifd_offsets, &image_file_header.endian);
//             ifds.extend(sub_ifds);
//         }
//         offset = ifd.next_ifd_offset;
//         ifds.insert(image_file_header.ifd_offset, ifd);
//     }

//     ifds
// }

// fn get_entry_values(buffer: &Vec<u8>, entry: &DirectoryEntry, endian: &Endian) -> EntryData {
//     let bytes_per_value = get_bytes_per_value_for_type(entry.data_type) as usize;
//     let total_used_bytes = bytes_per_value * entry.count;
//     match entry.count {
//         count if count > 1 => {
//             let mut values = Vec::with_capacity(entry.count as usize);
//             if total_used_bytes <= 4 {
//                 for i in 0..entry.count {
//                     values.push(get_entry_value(&entry.value_or_offset.to_be_bytes().to_vec(), entry, i * bytes_per_value, &Endian::Big));
//                 }
//             } else {
//                 for i in 0..entry.count {
//                     values.push(get_entry_value(buffer, entry, entry.value_or_offset as usize + i * bytes_per_value, endian));
//                 }
//             }
//             EntryData::Multiple(values)
//         },
//         count => {
//             EntryData::Single(DataType::Byte(8))
//         }
//     }    
// }





// fn parse_ifd_tree(buffer: &Vec<u8>, offsets: Vec<u32>, endian: &Endian) -> HashMap<usize, IFD> {
//     let mut ifds: HashMap<usize, IFD> = HashMap::new();
//     for i in offsets {
//         let ifd = parse_ifd(buffer, &(i as usize), endian);
//         if let Some(entry) = ifd.entries.get(&(Tag::SubIFD as u16)) {
//             let ifd_offsets = get_entry_values(buffer, entry, endian).to_vec().iter().map(|f| f.to_u32()).collect::<Vec<u32>>();
//             let sub_ifds = parse_ifd_tree(buffer, ifd_offsets, endian);
//             ifds.extend(sub_ifds);
//         }
//         ifds.insert(i as usize, ifd);
//     }
//     ifds
// }

// fn parse_ifd(buffer: &Vec<u8>, offset: usize, endian: &Endian) -> IFD {
//     let entry_count = get_value::short(buffer, offset, endian) as usize;    
//     let mut entries = HashMap::new();
//     for i in 0..entry_count {
//         let tag = get_value::short(buffer, offset + 2 + i * 12, endian);
//         let data_type = get_value::short(buffer, offset + 4 + i * 12, endian);
//         let count = get_value::long(buffer, offset + 6 + i * 12, endian) as usize;
//         let value_or_offset = get_value::long(buffer, offset + 10 + i * 12, endian);

//         entries.insert(tag.clone(), DirectoryEntry { tag, data_type: data_type, count, value_or_offset });
//     }
//     assert_eq!(get_value::long(buffer, offset + 2 + entry_count * 12, endian) as usize, 0);
//     IFD {
//         offset,
//         numb_of_entries: entry_count as u16,
//         entries,
//     }
// }

// fn get_entries_required_bytes(sub_ifd_entry: &DirectoryEntry) -> u32 {
//     let bytes_per_count = get_bytes_per_value_for_type(sub_ifd_entry.data_type) as u32;
//     bytes_per_count * sub_ifd_entry.count as u32
// }

pub(crate) fn get_bytes_per_value_for_type(data_type: u16) -> u16 {
    match data_type {
        1 | 2 | 6 | 7 => 1,
        3 | 8 => 2,
        4 | 9 | 11 => 4,
        5 | 10 | 12 => 8,
        _ => panic!("Received an invalid Directory Entry Type")
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn get_endian_big() {
//         let buffer = Vec::from([0x4Du8, 0x4D, 0xF0]);

//         assert_eq!(get_endian(&buffer), Endian::Big);
//     }

//     #[test]
//     fn get_endian_little() {
//         let buffer = Vec::from([0x49u8, 0x49, 0xF0]);

//         assert_eq!(get_endian(&buffer), Endian::Little);
//     }

//     #[test]
//     fn get_word_size_32() {
//         let buffer = Vec::from([0x4Du8, 0x4D, 42]);

//         assert_eq!(get_word_size(&buffer), WordSize::Thirtytwo);
//     }

//     #[test]
//     fn get_word_size_64() {
//         let buffer = Vec::from([0x49u8, 0x49, 43]);

//         assert_eq!(get_word_size(&buffer), WordSize::Sixtyfour);
//     }

//     #[test]
//     fn get_u8_at_idx_good() {
//         let buffer = Vec::from([200u8, 18, 0xF0]);

//         assert_eq!(get_u8_at_idx(&buffer, 1), 18);
//     }

//     #[test]
//     fn get_u16_at_idx_bigended_good() {
//         use crate::Endian;

//         let buffer = Vec::from([0xFFu8, 0xCC, 0x18, 0xCD]);

//         assert_eq!(get_u16_at_idx(&buffer, 1, &Endian::Big), 0xCC18u16);
//     }

//     #[test]
//     fn get_u16_at_idx_littleended_good() {
//         use crate::Endian;

//         let buffer = Vec::from([0xFFu8, 0xCC, 0x18, 0xCD]);

//         assert_eq!(get_u16_at_idx(&buffer, 1, &Endian::Little), 0x18CCu16);
//     }

//     #[test]
//     fn get_u32_at_idx_bigended_good() {
//         use crate::Endian;

//         let buffer = Vec::from([0xFFu8, 0xCC, 0x18, 0xCD, 0x97, 0xAA]);

//         assert_eq!(get_u32_at_idx(&buffer, 1, &Endian::Big), 0xCC18CD97u32);
//     }

//     #[test]
//     fn get_u32_at_idx_littleended_good() {
//         use crate::Endian;

//         let buffer = Vec::from([0xFFu8, 0xCC, 0x18, 0xCD, 0x97, 0xAA]);

//         assert_eq!(get_u32_at_idx(&buffer, 1, &Endian::Little), 0x97CD18CCu32);
//     }

//     #[test]
//     fn get_u64_at_idx_bigended_good() {
//         use crate::Endian;

//         let buffer = Vec::from([0xFFu8, 0xCC, 0x18, 0xCD, 0x97, 0xAA, 0xEC, 0x63, 0x5A, 0xA5]);

//         assert_eq!(get_u64_at_idx(&buffer, 1, &Endian::Big), 0xCC18CD97AAEC635Au64);
//     }

//     #[test]
//     fn get_u64_at_idx_littleended_good() {
//         use crate::Endian;

//         let buffer = Vec::from([0xFFu8, 0xCC, 0x18, 0xCD, 0x97, 0xAA, 0xEC, 0x63, 0x5A, 0xA5]);

//         assert_eq!(get_u64_at_idx(&buffer, 1, &Endian::Little), 0x5A63ECAA97CD18CCu64);
//     }
// }