#![allow(dead_code,unused_imports,unused_variables)]

use crate::jxl_decoder::JxlImageMetadata;
use crate::bit_reader::QuadDistributions::*;
use crate::bit_reader::BitStream;

#[derive(Debug,PartialEq,Eq)]
enum JxlFrameType {
    RegularFrame,
    LFFrame,
    ReferenceOnly,
    SkipProgressive
}
impl From<u8> for JxlFrameType {
    fn from(value: u8) -> Self {
        use JxlFrameType as E;
        match value {
            0 => E::RegularFrame,
            1 => E::LFFrame,
            2 => E::ReferenceOnly,
            3 => E::SkipProgressive,
            _ => unreachable!()
        }
    }
}

#[derive(Debug,PartialEq,Eq)]
enum JxlFrameEncoding {
    VarDCT, 
    Modular
}
impl From<u8> for JxlFrameEncoding {
    fn from(value: u8) -> Self {
        use JxlFrameEncoding as E;
        match value {
            0 => E::VarDCT,
            1 => E::Modular,
            _ => unreachable!()
        }
    }
}

#[derive(Debug)]
pub struct JxlFrame {
    frame_type: JxlFrameType,
    frame_encoding: JxlFrameEncoding
}

impl JxlFrame {
    pub fn read(bitstream: &mut BitStream, image_metadata: &JxlImageMetadata) -> Option<Self> {
        let all_default = bitstream.read_bool()?;
        println!("{:?}",all_default);
        let frame_type = if all_default {JxlFrameType::RegularFrame} else {JxlFrameType::from(bitstream.read_u8(2)?)};
        let frame_encoding = if all_default {JxlFrameEncoding::VarDCT} else {JxlFrameEncoding::from(bitstream.read_u8(1)?)};
        if frame_type != JxlFrameType::RegularFrame {
            todo!("Only regular frames are supported")
        }
        if frame_encoding != JxlFrameEncoding::Modular {
            todo!("Only Modular frames are supported")
        }

        Some(JxlFrame{
            frame_type,
            frame_encoding
        })
    }
}