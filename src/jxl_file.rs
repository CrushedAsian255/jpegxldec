#![allow(non_camel_case_types,dead_code,unused_imports)]

use std::io::Error as IoError;
use std::io::Read;
use std::fmt::Debug;

use crate::bit_reader::BitStream;
use crate::bit_reader::CapU32Distributions;

#[derive(Debug)]
pub struct JxlFile {
    boxes: Vec<JxlBox>
}

impl JxlFile {
    pub fn read<T: Read>(mut src: T) -> Result<Self,IoError> {
        let mut input_data: Vec<u8> = Vec::new();
        src.read_to_end(&mut input_data)?;

        let mut boxes: Vec<JxlBox> = Vec::new();
        loop {
            if input_data.len() == 0 { break; }
            else if input_data.starts_with(&[0xff, 0x0a]) {
                boxes.push(JxlBox {
                    box_type: JxlBoxType::JXL_RAW,
                    length: input_data.len() as u64,
                    data: input_data.to_owned()
                });
                break;
            } else {
                let mut box_len: u64 = u32::from_be_bytes(input_data.as_slice()[..4].try_into().unwrap()) as u64;
                let box_type: [u8; 4] = input_data.as_slice()[4..8].try_into().unwrap();
                let box_data: Vec<u8>;
                if box_len == 1 {
                    box_len = u64::from_be_bytes(input_data.as_slice()[8..16].try_into().unwrap());
                    box_data = input_data.drain(..(box_len as usize)).skip(16).collect::<Vec<u8>>();
                } else {
                    box_data = input_data.drain(..(box_len as usize)).skip(8).collect::<Vec<u8>>();
                }
                boxes.push(JxlBox {
                    box_type: match &box_type {
                        b"JXL " => JxlBoxType::JXL_SIGNATURE,
                        b"ftyp" => JxlBoxType::JXL_FILE_TYPE,
                        b"jxll" => JxlBoxType::JXL_LEVEL,
                        b"jumb" => JxlBoxType::JXL_JUMBF,
                        b"Exif" => JxlBoxType::JXL_EXIF,
                        b"xml " => JxlBoxType::JXL_XML,
                        b"brob" => JxlBoxType::JXL_BROTLI,
                        b"jxli" => JxlBoxType::JXL_INDEX,
                        b"jxlc" => JxlBoxType::JXL_CODESTREAM,
                        b"jxlp" => JxlBoxType::JXL_PARTIAL,
                        b"jbrd" => JxlBoxType::JXL_RECONSTRUCTION,
                        _ => panic!("Invalid box type: {:?}",String::from_utf8(box_type.into()).unwrap())
                    },
                    length: box_len,
                    data: box_data
                });
            } 
        };

        Ok(Self { boxes: boxes })
    }

    pub fn print_box_list(&self) -> Option<()> {
        for jxl_box in &self.boxes {
            println!("{:?}: {} bytes",jxl_box.box_type,jxl_box.length);
            use JxlBoxType as E;
            match jxl_box.box_type {
                E::JXL_RAW => println!("Raw codestream"),
                E::JXL_SIGNATURE => {
                    if jxl_box.data.as_slice() == b"\x0d\x0a\x87\x0a" {
                        println!("Valid signature");
                    } else {
                        panic!("Error: invalid JXL signature");
                    }
                },
                E::JXL_FILE_TYPE => {
                    if jxl_box.data.as_slice() == b"jxl \0\0\0\0jxl " {
                        println!("Valid file type");
                    } else {
                        panic!("Error: invalid JXL file type");
                    }
                },
                E::JXL_PARTIAL => {
                    let box_index = u32::from_be_bytes(jxl_box.data.as_slice()[0..4].try_into().unwrap());
                    let is_last_box: bool =  box_index & (1<<31) != 0;
                    println!("Index: {}, is last box: {}",box_index&!(1<<31),is_last_box);
                }
                _ => todo!()
            }
        }
        Some(())
    }

    pub fn get_image_data(&self) -> Vec<u8> {
        let mut output_vector = Vec::new();

        for jxl_box in &self.boxes {
            println!("{:?}: {} bytes",jxl_box.box_type,jxl_box.length);
            use JxlBoxType as E;
            match jxl_box.box_type {
                E::JXL_RAW => {
                    output_vector.extend(jxl_box.data.iter())
                },
                E::JXL_CODESTREAM | E::JXL_PARTIAL => todo!("not implemented"),
                _ => ()
            }
        }
        output_vector
    }
}


#[derive(Debug)]
pub enum JxlBoxType {
    JXL_RAW,            // If the file is a raw bitstream
    JXL_SIGNATURE,      // "JXL "
    JXL_FILE_TYPE,      // "ftyp"
    JXL_LEVEL,          // "jxll"
    JXL_JUMBF,          // "jumb"
    JXL_EXIF,           // "Exif"
    JXL_XML,            // "xml "
    JXL_BROTLI,         // "brob"
    JXL_INDEX,          // "jxli"
    JXL_CODESTREAM,     // "jxlc"
    JXL_PARTIAL,        // "jxlp"
    JXL_RECONSTRUCTION  // "jbrd"
}

#[derive(Debug)]
pub struct JxlBox {
    box_type: JxlBoxType,
    data: Vec<u8>,
    length: u64
}