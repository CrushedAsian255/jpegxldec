#![allow(non_camel_case_types,dead_code)]

use std::io::Error as IoError;
use std::io::Read;
use std::fmt::Debug;

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
                    length: input_data.len()-2,
                    data: input_data.to_owned().drain(2..).collect()
                });
                break;
            } else {
                let box_len = u32::from_be_bytes(input_data.as_slice()[..4].try_into().unwrap()) as usize;
                let box_type: [u8; 4] = input_data.as_slice()[4..8].try_into().unwrap();
                let box_data = input_data.drain(..box_len).skip(8).collect::<Vec<u8>>();
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

    pub fn print_box_list(&self) {
        for jxl_box in &self.boxes {
            println!("{:?}: {} bytes",jxl_box.box_type,jxl_box.length);
        }
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
    length: usize,
}