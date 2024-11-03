#![allow(dead_code,unused_imports,unused_variables)]

use crate::common::{ImageSize,unpack_signed};
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
pub struct JxlFramePasses {
    pass_count: u8,
    num_ds: u8,
    shifts: Vec<u8>,
    downsample: Vec<u8>,
    last_pass: Vec<u8>
}
impl JxlFramePasses {
    pub fn read(bitstream: &mut BitStream) -> Option<Self> {
        let pass_count = bitstream.read_quad_u32(RawValue(1), RawValue(2), RawValue(3), BitCountWithOffset(3, 4))? as u8;
        let num_ds = if pass_count == 1 { 0 } else { bitstream.read_quad_u32(RawValue(0), RawValue(1), RawValue(2), BitCountWithOffset(1, 3))? }  as u8; 
        let mut shifts: Vec<u8> = Vec::with_capacity(pass_count as usize);
        let mut downsample: Vec<u8> = Vec::with_capacity(num_ds as usize);
        let mut last_pass: Vec<u8> = Vec::with_capacity(num_ds as usize);
        for _ in 0..(pass_count-1) {
            shifts.push(bitstream.read_u8(2)?);
        }
        shifts.push(0);
        for _ in 0..num_ds {
            downsample.push(bitstream.read_quad_u32(RawValue(1), RawValue(2), RawValue(4), RawValue(8))? as u8);
        }
        for _ in 0..num_ds {
            last_pass.push(bitstream.read_quad_u32(RawValue(0), RawValue(1), RawValue(2), BitCount(3))? as u8);
        }

        Some(JxlFramePasses {
            pass_count,
            num_ds,
            shifts,
            downsample,
            last_pass
        })
    }
}

#[derive(Debug)]
pub struct JxlFrameCropInfo {
    pub width: u32,
    pub height: u32,
    pub x0: i32,
    pub y0: i32    
}

#[derive(Debug)]
pub struct JxlFrameHeader {
    frame_type: JxlFrameType,
    frame_encoding: JxlFrameEncoding,
    flags: JxlFrameFlags,
    ycbcr: bool,
    jpeg_upscaling: [u8;3],
    upsampling: u8,
    ec_upscaling: Vec<u8>,
    modular_group_size: Option<u16>,
    x_qm_scale: u8,
    b_qm_scale: u8,
    passes: JxlFramePasses,
    lf_level: Option<u8>,
    crop_info: Option<JxlFrameCropInfo>
}

#[derive(Debug)]
pub struct JxlFrame {
    pub header: JxlFrameHeader
}

impl JxlFrameHeader {
    pub fn read(bitstream: &mut BitStream, image_metadata: &JxlImageMetadata) -> Option<Self> {
        let all_default = bitstream.read_bool()?;
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
        let jpeg_upscaling = if !ycbcr || flags.use_lf_frame { [1,1,1] } else {[
            bitstream.read_u8(2)?,
            bitstream.read_u8(2)?,
            bitstream.read_u8(2)?
        ]};
        let upsampling = if all_default || flags.use_lf_frame { 1 } else {bitstream.read_quad_u32(RawValue(1), RawValue(2), RawValue(4), RawValue(8))?} as u8;
        let mut ec_upscaling = Vec::new();
        for _ in 0..image_metadata.extra_channels.len() {
            ec_upscaling.push(bitstream.read_quad_u32(RawValue(1), RawValue(2), RawValue(4), RawValue(8))? as u8);
        }
        let modular_group_size = if frame_encoding != JxlFrameEncoding::Modular { None } else { Some(128 << bitstream.read_u16(2)?) };
        let d_xqms = if image_metadata.xyb_encoded && frame_encoding == JxlFrameEncoding::VarDCT {3} else {2};
        let x_qm_scale = if all_default || !image_metadata.xyb_encoded || frame_encoding != JxlFrameEncoding::VarDCT { d_xqms } else {bitstream.read_u8(3)?};
        let b_qm_scale = if all_default || !image_metadata.xyb_encoded || frame_encoding != JxlFrameEncoding::VarDCT { 2 } else {bitstream.read_u8(3)?};
        
        let passes = JxlFramePasses::read(bitstream)?;

        let lf_level = if frame_type == JxlFrameType::LFFrame {Some(1 + bitstream.read_u8(2)?)} else {None};

        let crop_info = if all_default || flags.use_lf_frame || !(bitstream.read_bool()?) {None} else {
            let ux0 = bitstream.read_quad_u32(BitCount(8), BitCountWithOffset(11, 8), BitCountWithOffset(14, 2304), BitCountWithOffset(30, 18688))?;
            let uy0 = bitstream.read_quad_u32(BitCount(8), BitCountWithOffset(11, 8), BitCountWithOffset(14, 2304), BitCountWithOffset(30, 18688))?;
            let width = bitstream.read_quad_u32(BitCount(8), BitCountWithOffset(11, 8), BitCountWithOffset(14, 2304), BitCountWithOffset(30, 18688))?;
            let height = bitstream.read_quad_u32(BitCount(8), BitCountWithOffset(11, 8), BitCountWithOffset(14, 2304), BitCountWithOffset(30, 18688))?;
            Some(JxlFrameCropInfo {
                x0: unpack_signed(ux0),
                y0: unpack_signed(uy0),
                width,
                height
            })
        };

        let normal_frame = frame_type == JxlFrameType::RegularFrame || frame_type == JxlFrameType::SkipProgressive;


        Some(JxlFrameHeader {
            frame_type,
            frame_encoding,
            flags,
            ycbcr,
            jpeg_upscaling,
            upsampling,
            ec_upscaling,
            modular_group_size,
            x_qm_scale,
            b_qm_scale,
            passes,
            lf_level,
            crop_info
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