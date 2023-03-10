use std::{collections::HashMap, hash::Hash};

// pub(crate) fn is_dng(bytes: &[u8]) -> bool {
//     if u16::from_be_bytes([bytes[0], bytes[1]]) == 0xFFD8 as u16 {
//         true
//     } else {
//         false
//     }
// }
use crate::{ Endedness, WordSize, ImageFileHeader, IFD, DirectoryEntry, tags::Tag};

// See TIFF6, Section 2, Image File Header
pub(crate) fn get_endedness(buffer: &Vec<u8>,) -> Endedness {
    use Endedness::*;
    match get_u16_at_idx(buffer, 0, &Big) {
        0x4949 => Little,
        0x4D4D => Big,
        _ => panic!("Endedness bytes aren't valid!!!")
    }
}

pub(crate) fn get_word_size(buffer: &Vec<u8>,) -> WordSize {
    use WordSize::*;
    match get_u8_at_idx(buffer, 2) {
        42 => Thirtytwo,
        43 => Sixtyfour,
        _ => panic!("Endedness bytes aren't valid!!!")
    }
}

pub(crate) fn get_u32_at_idx_from_n_bytes(buffer: &Vec<u8>, idx: usize, endedness: &Endedness, bytes_per_value: usize) -> u32 {
    let bytes = &buffer[idx..idx + bytes_per_value];

    let mut output: u32 = 0;

    use Endedness::*;
    match endedness {
        Big => {
            for i in 0..bytes_per_value {
                output = output << 8 | (bytes[i] as u32);
            }
        },
        Little => {
            for i in 1..=bytes_per_value {
                output = output << 8 | (bytes[bytes_per_value - i] as u32);
            }
        }
    }
    output
}

pub(crate) fn get_u8_at_idx(buffer: &Vec<u8>, idx: usize) -> u8 {
    buffer[idx] as u8
}

pub(crate) fn get_u16_at_idx(buffer: &Vec<u8>, idx: usize, endedness: &Endedness) -> u16 {
    let bytes = &buffer[idx..idx + 2];

    use Endedness::*;
    match endedness {
        Big => {
            ((bytes[0] as u16) << 8) +
            ((bytes[1] as u16) << 0)
        },
        Little => {
            ((bytes[0] as u16) << 0) +
            ((bytes[1] as u16) << 8)
        }
    }
}

pub(crate) fn get_u32_at_idx(buffer: &Vec<u8>, idx: usize, endedness: &Endedness) -> u32 {
    let bytes = &buffer[idx..idx + 4];

    use Endedness::*;
    match endedness {
        Big => {
            ((bytes[0] as u32) << 24) +
            ((bytes[1] as u32) << 16) +
            ((bytes[2] as u32) <<  8) +
            ((bytes[3] as u32) <<  0)
        },
        Little => {
            ((bytes[0] as u32) <<  0) +
            ((bytes[1] as u32) <<  8) +
            ((bytes[2] as u32) << 16) +
            ((bytes[3] as u32) << 24)
        }
    }
}

pub(crate) fn get_u64_at_idx(buffer: &Vec<u8>, idx: usize, endedness: &Endedness) -> u64 {
    let bytes = &buffer[idx..idx + 8];

    use Endedness::*;
    match endedness {
        Big => {
            ((bytes[0] as u64) << 56) +
            ((bytes[1] as u64) << 48) +
            ((bytes[2] as u64) << 40) +
            ((bytes[3] as u64) << 32) +
            ((bytes[4] as u64) << 24) +
            ((bytes[5] as u64) << 16) +
            ((bytes[6] as u64) <<  8) +
            ((bytes[7] as u64) <<  0)
        },
        Little => {
            ((bytes[0] as u64) <<  0) +
            ((bytes[1] as u64) <<  8) +
            ((bytes[2] as u64) << 16) +
            ((bytes[3] as u64) << 24) +
            ((bytes[4] as u64) << 32) +
            ((bytes[5] as u64) << 40) +
            ((bytes[6] as u64) << 48) +
            ((bytes[7] as u64) << 56)
        }
    }
}

pub(crate) fn parse_ifds(buffer: &Vec<u8>, image_file_header: &ImageFileHeader) -> HashMap<usize, IFD> {
    let mut ifds = HashMap::new();

    let mut offset = image_file_header.ifd_offset;
    while offset > 0 {
        let ifd = parse_ifd(buffer, &image_file_header.ifd_offset, &image_file_header.endedness);
        if let Some(entry) = ifd.entries.get(&(Tag::SubIFD as u16)) {
            let ifd_offsets = get_values_as_u32(buffer, entry, &image_file_header.endedness);
            let sub_ifds = parse_ifd_tree(buffer, ifd_offsets, &image_file_header.endedness);
            ifds.extend(sub_ifds);
        }
        offset = ifd.next_ifd_offset;
        ifds.insert(image_file_header.ifd_offset, ifd);
    }

    ifds
}

fn parse_ifd_tree(buffer: &Vec<u8>, offsets: Vec<u32>, endedness: &Endedness) -> HashMap<usize, IFD> {
    let mut ifds: HashMap<usize, IFD> = HashMap::new();

    for i in offsets {
        let ifd = parse_ifd(buffer, &(i as usize), endedness);
        if let Some(entry) = ifd.entries.get(&(Tag::SubIFD as u16)) {
            let ifd_offsets = get_values_as_u32(buffer, entry, endedness);
            let sub_ifds = parse_ifd_tree(buffer, ifd_offsets, endedness);
            ifds.extend(sub_ifds);
        }
        ifds.insert(i as usize, ifd);
    }

    ifds
}

fn parse_ifd(buffer: &Vec<u8>, offset: &usize, endedness: &Endedness) -> IFD {
    let entry_count = get_u16_at_idx(buffer, *offset, endedness) as usize;    
    let mut entries = HashMap::new();
    for i in 0..entry_count {
        let tag = get_u16_at_idx(buffer, offset + 2 + i * 12, endedness);
        let entry_type = get_u16_at_idx(buffer, offset + 4 + i * 12, endedness);
        let count = get_u32_at_idx(buffer, offset + 6 + i * 12, endedness) as usize;
        let value_or_offset = get_u32_at_idx(buffer, offset + 10 + i * 12, endedness);

        entries.insert(tag.clone(), DirectoryEntry { tag, entry_type, count, value_or_offset });
    }
    IFD {
        numb_of_entries: entry_count as u16,
        entries,
        next_ifd_offset: get_u32_at_idx(buffer, offset + 2 + entry_count * 12, endedness) as usize,
    }
}

fn get_values_as_u32(buffer: &Vec<u8>, sub_ifd_entry: &DirectoryEntry, endedness: &Endedness) -> Vec<u32> {
    let mut values = Vec::with_capacity(sub_ifd_entry.count as usize);
    let bytes_per_value = get_bytes_per_value_for_type(sub_ifd_entry.entry_type) as usize;
    if get_entries_required_bytes(sub_ifd_entry) <= 4 {
        for i in 0..sub_ifd_entry.count {
            values.push(get_u32_at_idx_from_n_bytes(
                &get_array_from_u32(sub_ifd_entry.value_or_offset, endedness).to_vec(), 
                i * bytes_per_value, 
                endedness, 
                bytes_per_value))
        }
    } else {
        for i in 0..sub_ifd_entry.count {
            values.push(get_u32_at_idx_from_n_bytes(
                buffer, 
                sub_ifd_entry.value_or_offset as usize + i * bytes_per_value, 
                endedness, 
                bytes_per_value))
        }
    }
    values
}

fn get_entries_required_bytes(sub_ifd_entry: &DirectoryEntry) -> u32 {
    let bytes_per_count = get_bytes_per_value_for_type(sub_ifd_entry.entry_type) as u32;
    bytes_per_count * sub_ifd_entry.count as u32
}

fn get_bytes_per_value_for_type(entry_type: u16) -> u16 {
    match entry_type {
        1 | 2 | 6 | 7 => 1,
        3 | 8 => 2,
        4 | 9 | 11 => 4,
        5 | 10 | 12 => 8,
        _ => panic!("Received an invalid Directory Entry Type")
    }
}

fn get_array_from_u32(number: u32, endedness: &Endedness) -> [u8; 4] {
    match endedness {
        Endedness::Big => {
            [
            ((number >> 24) & 0xF) as u8,
            ((number >> 16) & 0xF) as u8,
            ((number >>  8) & 0xF) as u8,
            ((number >>  0) & 0xF) as u8,
            ]
        },
        Endedness::Little => {
            [
            ((number >>  0) & 0xF) as u8,
            ((number >>  8) & 0xF) as u8,
            ((number >> 16) & 0xF) as u8,
            ((number >> 24) & 0xF) as u8,
            ]
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_endedness_big() {
        let buffer = Vec::from([0x4Du8, 0x4D, 0xF0]);

        assert_eq!(get_endedness(&buffer), Endedness::Big);
    }

    #[test]
    fn get_endedness_little() {
        let buffer = Vec::from([0x49u8, 0x49, 0xF0]);

        assert_eq!(get_endedness(&buffer), Endedness::Little);
    }

    #[test]
    fn get_word_size_32() {
        let buffer = Vec::from([0x4Du8, 0x4D, 42]);

        assert_eq!(get_word_size(&buffer), WordSize::Thirtytwo);
    }

    #[test]
    fn get_word_size_64() {
        let buffer = Vec::from([0x49u8, 0x49, 43]);

        assert_eq!(get_word_size(&buffer), WordSize::Sixtyfour);
    }

    #[test]
    fn get_u8_at_idx_good() {
        let buffer = Vec::from([200u8, 18, 0xF0]);

        assert_eq!(get_u8_at_idx(&buffer, 1), 18);
    }

    #[test]
    fn get_u16_at_idx_bigended_good() {
        use crate::Endedness;

        let buffer = Vec::from([0xFFu8, 0xCC, 0x18, 0xCD]);

        assert_eq!(get_u16_at_idx(&buffer, 1, &Endedness::Big), 0xCC18u16);
    }

    #[test]
    fn get_u16_at_idx_littleended_good() {
        use crate::Endedness;

        let buffer = Vec::from([0xFFu8, 0xCC, 0x18, 0xCD]);

        assert_eq!(get_u16_at_idx(&buffer, 1, &Endedness::Little), 0x18CCu16);
    }

    #[test]
    fn get_u32_at_idx_bigended_good() {
        use crate::Endedness;

        let buffer = Vec::from([0xFFu8, 0xCC, 0x18, 0xCD, 0x97, 0xAA]);

        assert_eq!(get_u32_at_idx(&buffer, 1, &Endedness::Big), 0xCC18CD97u32);
    }

    #[test]
    fn get_u32_at_idx_littleended_good() {
        use crate::Endedness;

        let buffer = Vec::from([0xFFu8, 0xCC, 0x18, 0xCD, 0x97, 0xAA]);

        assert_eq!(get_u32_at_idx(&buffer, 1, &Endedness::Little), 0x97CD18CCu32);
    }

    #[test]
    fn get_u64_at_idx_bigended_good() {
        use crate::Endedness;

        let buffer = Vec::from([0xFFu8, 0xCC, 0x18, 0xCD, 0x97, 0xAA, 0xEC, 0x63, 0x5A, 0xA5]);

        assert_eq!(get_u64_at_idx(&buffer, 1, &Endedness::Big), 0xCC18CD97AAEC635Au64);
    }

    #[test]
    fn get_u64_at_idx_littleended_good() {
        use crate::Endedness;

        let buffer = Vec::from([0xFFu8, 0xCC, 0x18, 0xCD, 0x97, 0xAA, 0xEC, 0x63, 0x5A, 0xA5]);

        assert_eq!(get_u64_at_idx(&buffer, 1, &Endedness::Little), 0x5A63ECAA97CD18CCu64);
    }
}