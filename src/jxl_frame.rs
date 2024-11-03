#![allow(dead_code,unused_imports,unused_variables)]

use crate::jxl_image::JxlImageMetadata;
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
struct JxlFrameFlags {
    use_noise: bool,
    use_patches: bool,
    use_splines: bool,
    use_lf_frame: bool,
    use_adaptive_lf_smoothing: bool
}
impl From<u64> for JxlFrameFlags {
    fn from(value: u64) -> Self {
        Self {
            use_noise: (value & 0x1) != 0,
            use_patches: (value & 0x2) != 0,
            use_splines: (value & 0x10) != 0,
            use_lf_frame: (value & 0x20) != 0,
            use_adaptive_lf_smoothing: (value & 0x80) == 0
        }
    }
}

#[derive(Debug)]
pub struct JxlFrameHeader {
    frame_type: JxlFrameType,
    frame_encoding: JxlFrameEncoding,
    flags: JxlFrameFlags,
    ycbcr: bool
}

#[derive(Debug)]
pub struct JxlFrame {
    pub header: JxlFrameHeader
}

impl JxlFrameHeader {
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
        let flags = JxlFrameFlags::from(if all_default { 0u64 } else {bitstream.read_var_u64()?});
        let ycbcr = if all_default || image_metadata.xyb_encoded { false } else { bitstream.read_bool()? };
        Some(JxlFrameHeader {
            frame_type,
            frame_encoding,
            flags,
            ycbcr
        })
    }
}

impl JxlFrame {
    pub fn read(bitstream: &mut BitStream, image_metadata: &JxlImageMetadata) -> Option<Self> {
        Some(Self {
            header: JxlFrameHeader::read(bitstream, image_metadata)?
        })
    }
}