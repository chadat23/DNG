use crate::Endian;

pub(crate) fn byte(buffer: &Vec<u8>, offset: usize) -> u8 {
    buffer[offset] as u8
}

pub(crate) fn ascii(buffer: &Vec<u8>, offset: usize) -> u8 {
    byte(buffer, offset)
}

pub(crate) fn short(buffer: &Vec<u8>, offset: usize, endian: &Endian) -> u16 {
    let bytes = &buffer[offset..offset + 2];

    let a = u16::from_be_bytes(bytes.try_into().unwrap());

    use Endian::*;
    match endian {
        Big => {
            u16::from_be_bytes(bytes.try_into().unwrap())
        },
        Little => {
            u16::from_le_bytes(bytes.try_into().unwrap())
        }
    }
}

pub(crate) fn long(buffer: &Vec<u8>, offset: usize, endian: &Endian) -> u32 {
    let bytes = &buffer[offset..offset + 4];

    use Endian::*;
    match endian {
        Big => {
            u32::from_be_bytes(bytes.try_into().unwrap())
        },
        Little => {
            u32::from_le_bytes(bytes.try_into().unwrap())
        }
    }
}

pub(crate) fn rational(buffer: &Vec<u8>, offset: usize, endian: &Endian) -> [u32; 2] {
    let bytes0 = &buffer[offset..offset + 4];
    let bytes1 = &buffer[offset + 4..offset + 8];

    use Endian::*;
    match endian {
        Big => {
            [
                u32::from_be_bytes(bytes0.try_into().unwrap()),
                u32::from_be_bytes(bytes1.try_into().unwrap()),
            ]
        },
        Little => {
            [
                u32::from_le_bytes(bytes0.try_into().unwrap()),
                u32::from_le_bytes(bytes1.try_into().unwrap()),
            ]
        }
    }
}

pub(crate) fn sbyte(buffer: &Vec<u8>, offset: usize) -> i8 {
    buffer[offset] as i8
}

pub(crate) fn undefined(buffer: &Vec<u8>, offset: usize) -> u8 {
    byte(buffer, offset)
}

pub(crate) fn sshort(buffer: &Vec<u8>, offset: usize, endian: &Endian) -> i16 {
    let bytes = &buffer[offset..offset + 2];

    use Endian::*;
    match endian {
        Big => {
            i16::from_be_bytes(bytes.try_into().unwrap())
        },
        Little => {
            i16::from_le_bytes(bytes.try_into().unwrap())
        }
    }
}

pub(crate) fn slong(buffer: &Vec<u8>, offset: usize, endian: &Endian) -> i32 {
    let bytes = &buffer[offset..offset + 4];

    use Endian::*;
    match endian {
        Big => {
            i32::from_be_bytes(bytes.try_into().unwrap())
        },
        Little => {
            i32::from_le_bytes(bytes.try_into().unwrap())
        }
    }
}

pub(crate) fn rsational(buffer: &Vec<u8>, offset: usize, endian: &Endian) -> [i32; 2] {
    let bytes0 = &buffer[offset..offset + 4];
    let bytes1 = &buffer[offset + 4..offset + 8];

    use Endian::*;
    match endian {
        Big => {
            [
                i32::from_be_bytes(bytes0.try_into().unwrap()),
                i32::from_be_bytes(bytes1.try_into().unwrap()),
            ]
        },
        Little => {
            [
                i32::from_le_bytes(bytes0.try_into().unwrap()),
                i32::from_le_bytes(bytes1.try_into().unwrap()),
            ]
        }
    }
}

pub(crate) fn float(buffer: &Vec<u8>, offset: usize, endian: &Endian) -> f32 {
    let bytes = &buffer[offset..offset + 4];

    use Endian::*;
    match endian {
        Big => {
            f32::from_be_bytes(bytes.try_into().unwrap())
        },
        Little => {
            f32::from_le_bytes(bytes.try_into().unwrap())
        }
    }
}

pub(crate) fn double(buffer: &Vec<u8>, offset: usize, endian: &Endian) -> f64 {
    let bytes = &buffer[offset..offset + 8];

    use Endian::*;
    match endian {
        Big => {
            f64::from_be_bytes(bytes.try_into().unwrap())
        },
        Little => {
            f64::from_le_bytes(bytes.try_into().unwrap())
        }
    }
}

// fn byte(buffer: &Vec<u8>, offset: usize) -> u8 {
//     buffer[offset] as u8
// }

// fn ascii(buffer: &Vec<u8>, offset: usize) -> u8 {
//     get_byte(buffer, offset)
// }

// fn short(buffer: &Vec<u8>, offset: usize, endian: &Endedness) -> u16 {
//     let bytes = &buffer[offset..offset + 2];

//     use Endedness::*;
//     match endian {
//         Big => {
//             ((bytes[0] as u16) << 8) +
//             ((bytes[1] as u16) << 0)
//         },
//         Little => {
//             ((bytes[0] as u16) << 0) +
//             ((bytes[1] as u16) << 8)
//         }
//     }
// }

// fn long(buffer: &Vec<u8>, offset: usize, endian: &Endedness) -> u32 {
//     let bytes = &buffer[offset..offset + 4];

//     use Endedness::*;
//     match endian {
//         Big => {
//             ((bytes[0] as u32) << 24) +
//             ((bytes[1] as u32) << 16) +
//             ((bytes[2] as u32) <<  8) +
//             ((bytes[3] as u32) <<  0)
//         },
//         Little => {
//             ((bytes[0] as u32) <<  0) +
//             ((bytes[1] as u32) <<  8) +
//             ((bytes[2] as u32) << 16) +
//             ((bytes[3] as u32) << 24)
//         }
//     }
// }

// fn rational(buffer: &Vec<u8>, idx: usize, endian: &Endedness) -> [u32; 2] {
//     let bytes1 = &buffer[idx..idx + 8];

//     use Endedness::*;
//     match endian {
//         Big => {
//             [((bytes[0] as u64) << 56) +
//              ((bytes[1] as u64) << 48) +
//              ((bytes[2] as u64) << 40) +
//              ((bytes[3] as u64) << 32),
//              ((bytes[4] as u64) << 24) +
//              ((bytes[5] as u64) << 16) +
//              ((bytes[6] as u64) <<  8) +
//              ((bytes[7] as u64) <<  0)]
//         },
//         Little => {
//             [((bytes[0] as u64) <<  0) +
//              ((bytes[1] as u64) <<  8) +
//              ((bytes[2] as u64) << 16) +
//              ((bytes[3] as u64) << 24),
//              ((bytes[4] as u64) << 32) +
//              ((bytes[5] as u64) << 40) +
//              ((bytes[6] as u64) << 48) +
//              ((bytes[7] as u64) << 56)]
//         }
//     }
// }

// fn sbyte(buffer: &Vec<u8>, offset: usize) -> i8 {
//     buffer[offset] as i8
// }

// fn undefined(buffer: &Vec<u8>, offset: usize) -> i8 {
//     get_byte(buffer, offset)
// }

// fn sshort(buffer: &Vec<u8>, offset: usize, endian: &Endedness) -> u16 {
//     let bytes = &buffer[offset..offset + 2];

//     use Endedness::*;
//     match endian {
//         Big => {
//             ((bytes[0] as u16) << 8) +
//             ((bytes[1] as u16) << 0) as i16
//         },
//         Little => {
//             ((bytes[0] as u16) << 0) +
//             ((bytes[1] as u16) << 8) as i16
//         }
//     }
// }

// fn slong(buffer: &Vec<u8>, offset: usize, endian: &Endedness) -> u32 {
//     let bytes = &buffer[offset..offset + 4];

//     use Endedness::*;
//     match endian {
//         Big => {
//             ((bytes[0] as u32) << 24) +
//             ((bytes[1] as u32) << 16) +
//             ((bytes[2] as u32) <<  8) +
//             ((bytes[3] as u32) <<  0) as i32
//         },
//         Little => {
//             ((bytes[0] as u32) <<  0) +
//             ((bytes[1] as u32) <<  8) +
//             ((bytes[2] as u32) << 16) +
//             ((bytes[3] as u32) << 24) as i32
//         }
//     }
// }

// fn rsational(buffer: &Vec<u8>, idx: usize, endian: &Endedness) -> [u32; 2] {
//     let bytes = &buffer[idx..idx + 8];

//     use Endedness::*;
//     match endian {
//         Big => {
//             [((bytes[0] as u64) << 56) +
//              ((bytes[1] as u64) << 48) +
//              ((bytes[2] as u64) << 40) +
//              ((bytes[3] as u64) << 32) as i32,
//              ((bytes[4] as u64) << 24) +
//              ((bytes[5] as u64) << 16) +
//              ((bytes[6] as u64) <<  8) +
//              ((bytes[7] as u64) <<  0) as i32]
//         },
//         Little => {
//             [((bytes[0] as u64) <<  0) +
//              ((bytes[1] as u64) <<  8) +
//              ((bytes[2] as u64) << 16) +
//              ((bytes[3] as u64) << 24),
//              ((bytes[4] as u64) << 32) +
//              ((bytes[5] as u64) << 40) +
//              ((bytes[6] as u64) << 48) +
//              ((bytes[7] as u64) << 56)]
//         }
//     }
// }
