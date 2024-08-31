use crate::bit_reader::BitStream;
use crate::bit_reader::QuadDistributions;
use crate::bit_reader::QuadDistributions::*;
use crate::jxl_file::JxlFile;
use crate::pixel_array::PixelArray;

#[derive(Debug,PartialEq)]
struct JxlImageSize {
    width: u32,
    height: u32
}

impl JxlImageSize {
    pub fn read(bitstream: &mut BitStream) -> Option<Self> {
        let div8 = bitstream.read_bool()?;
        let height = if div8 {
            (bitstream.read_u8(5)? + 1) as u32 * 8
        } else {
            bitstream.read_quad_u32(
                BitCountWithOffset(9,1), 
                BitCountWithOffset(13,1),
                 BitCountWithOffset(18,1), 
                 BitCountWithOffset(30,1)
            )?
        };
        let ratio = bitstream.read_u8(3)?;
        let width = match ratio {
            0 => if div8 {
                (bitstream.read_u8(5)? + 1) as u32 * 8
            } else {
                bitstream.read_quad_u32(
                    BitCountWithOffset(9,1), 
                    BitCountWithOffset(13,1),
                     BitCountWithOffset(18,1), 
                     BitCountWithOffset(30,1)
                )?
            },
            1 => height,
            2 => (height * 6) / 5,
            3 => (height * 4) / 3,
            4 => (height * 3) / 2,
            5 => (height * 16) / 9,
            6 => (height * 5) / 4,
            7 => height * 2,
            _ => unreachable!()
        };
        Some(JxlImageSize { width, height })
    }
}

#[derive(Debug)]
enum JxlOrientation {
    Normal,
    Rotate90,
    Rotate180,
    Rotate270,
    HorizontalFlip,
    VerticalFlip,
    Rotate90HorizontalFlip,
    HorizontalFlipRotate90
}

impl From<u8> for JxlOrientation {
    fn from(value: u8) -> Self {
        use JxlOrientation as E;
        match value {
            0 => E::Normal,
            1 => E::HorizontalFlip,
            2 => E::Rotate180,
            3 => E::VerticalFlip,
            4 => E::Rotate90HorizontalFlip,
            5 => E::Rotate90,
            6 => E::HorizontalFlipRotate90,
            7 => E::Rotate270,
            _ => unreachable!()
        }
    }
}

impl JxlImageSize {
    pub fn read_preview(bitstream: &mut BitStream) -> Option<Self> {
        let div8 = bitstream.read_bool()?;
        let height = if div8 {
            bitstream.read_quad_u32(
                RawValue(16),
                RawValue(32),
                BitCountWithOffset(5,1),
                BitCountWithOffset(9,33)
            )? * 8
        } else {
            bitstream.read_quad_u32(
                BitCountWithOffset(6,1), 
                BitCountWithOffset(8,65),
                 BitCountWithOffset(10,321), 
                 BitCountWithOffset(12,1345)
            )?
        };
        let ratio = bitstream.read_u8(3)?;
        let width = match ratio {
            0 => if div8 {
                bitstream.read_quad_u32(
                    RawValue(16),
                    RawValue(32),
                    BitCountWithOffset(5,1),
                    BitCountWithOffset(9,33)
                )? * 8
            } else {
                bitstream.read_quad_u32(
                    BitCountWithOffset(6,1), 
                    BitCountWithOffset(8,65),
                     BitCountWithOffset(10,321), 
                     BitCountWithOffset(12,1345)
                )?
            },
            1 => height,
            2 => (height * 6) / 5,
            3 => (height * 4) / 3,
            4 => (height * 3) / 2,
            5 => (height * 16) / 9,
            6 => (height * 5) / 4,
            7 => height * 2,
            _ => unreachable!()
        };
        Some(JxlImageSize { width, height })
    }
}


#[derive(Debug)]
struct JxlAnimationInfo {
    tps_numerator: u32,
    tps_denominator: u32,
    loop_count: u32,
    has_timecodes: bool
}

impl JxlAnimationInfo {
    pub fn read(bitstream: &mut BitStream) -> Option<Self> {
        let tps_numerator = bitstream.read_quad_u32(
            RawValue(100), 
            RawValue(1000), 
            BitCountWithOffset(10, 1),
            BitCountWithOffset(30, 1)
        )?;
        let tps_denominator = bitstream.read_quad_u32(
            RawValue(1), 
            RawValue(1001), 
            BitCountWithOffset(8, 1),
            BitCountWithOffset(10, 1)
        )?;
        let loop_count = bitstream.read_quad_u32(
            RawValue(0), 
            BitCount(3),
            BitCount(16), 
            BitCount(32)
        )?;
        let has_timecodes = bitstream.read_bool()?;
        Some(JxlAnimationInfo {
            tps_numerator,
            tps_denominator,
            loop_count,
            has_timecodes
        })
    }
}

#[derive(Debug)]
enum JxlBitDepth {
    Integer {
        bits: u8
    },
    Float {
        bits: u8,
        exp_bits: u8
    }
}

impl JxlBitDepth {
    pub fn read(bitstream: &mut BitStream) -> Option<Self> {
        match bitstream.read_bool()? {
            false => {
                Some(Self::Integer { bits: bitstream.read_quad_u32(
                    RawValue(8),
                    RawValue(10),
                    RawValue(12),
                    BitCountWithOffset(6, 1)
                )? as u8 })
            },
            true => {
                let bits = bitstream.read_quad_u32(
                    RawValue(32),
                    RawValue(16),
                    RawValue(24),
                    BitCountWithOffset(6, 1)
                )? as u8;
                let exp_bits = bitstream.read_u8(4)? + 1;
                Some(Self::Float { bits, exp_bits })
            }
        }
    } 
}

#[derive(Debug)]
struct JxlExtraChannel {}

#[derive(Debug)]
struct JxlColourEncoding {}
impl JxlColourEncoding {
    pub fn read(bitstream: &mut BitStream) -> Option<Self> {
        let all_default = bitstream.read_bool()?;
        if all_default {
            return Some(Self{});
        } else {
            todo!("Not implemented - colour management is hard");
        }
    }
}

#[derive(Debug)]
struct JxlToneMapping {}

#[derive(Debug)]
struct JxlExtensions {
    extensions: Vec<u64>
}
impl JxlExtensions {
    pub fn read(bitstream: &mut BitStream) -> Option<Self> {
        let extensions_bitmap = bitstream.read_var_u64()?;
        if extensions_bitmap == 0 {
            Some(Self{extensions: Vec::new()})
        } else {
            todo!("Not implemented {:#}",extensions_bitmap)
        }
    }
}

#[derive(Debug)]
struct JxlImageMetadata {
    orientation: JxlOrientation,
    intrinsic_size: Option<JxlImageSize>,
    preview_size: Option<JxlImageSize>,
    animation_info: Option<JxlAnimationInfo>,
    bit_depth: JxlBitDepth,
    modular_16bit: bool,
    extra_channels: Vec<JxlExtraChannel>,
    xyb_encoded: bool,
    colour_encoding: JxlColourEncoding,
    tone_mapping: Option<JxlToneMapping>,
    extensions: Option<JxlExtensions>
}

impl JxlImageMetadata {
    pub fn read(bitstream: &mut BitStream) -> Option<Self> {
        let all_default = bitstream.read_bool()?;
        let extra_fields = if all_default { false } else {bitstream.read_bool()?};
        println!("Using all default options: {}\nExtra fields present: {}",all_default,extra_fields);
        let orientation = if extra_fields { JxlOrientation::from(bitstream.read_u8(3)?) } else { JxlOrientation::Normal };
        let intrinsic_size = if !extra_fields { None } else {
            if bitstream.read_bool()? {
                Some(JxlImageSize::read(bitstream).unwrap())
            } else { None }
        };
        let preview_size = if !extra_fields { None } else {
            if bitstream.read_bool()? {
                Some(JxlImageSize::read_preview(bitstream).unwrap())
            } else { None }
        };
        let animation_info = if !extra_fields { None } else {
            if bitstream.read_bool()? {
                Some(JxlAnimationInfo::read(bitstream).unwrap())
            } else { None }
        };
        let bit_depth = JxlBitDepth::read(bitstream)?;
        let modular_16bit = bitstream.read_bool()?;
        let extra_channel_count = bitstream.read_quad_u32(
            RawValue(0),
            RawValue(1),
            BitCountWithOffset(4, 2),
            BitCountWithOffset(12, 1)
        )?;
        let extra_channels: Vec<JxlExtraChannel> = Vec::with_capacity(extra_channel_count as usize);
        for _ in 0..extra_channel_count {
            todo!("Read extra channels");
        }
        let xyb_encoded = bitstream.read_bool()?;
        let colour_encoding = JxlColourEncoding::read(bitstream)?;
        let tone_mapping = if extra_fields { todo!("Not implemented") } else { None };
        let extensions = if all_default {
            None
        } else {
            let extensions_ = JxlExtensions::read(bitstream)?;
            if extensions_.extensions.len() == 0 { None } else { Some(extensions_) }
        };
        let default_m = bitstream.read_bool()?;
        if default_m {
            if xyb_encoded {
                todo!("Custom XYB Matrix")
            }
            let cw_mask = bitstream.read_u8(3)?;
            if cw_mask != 0 {
                todo!("CW Mask {:#}",cw_mask);
            }
        }
        Some(JxlImageMetadata {
            orientation,
            intrinsic_size,
            preview_size,
            animation_info,
            bit_depth,
            modular_16bit,
            extra_channels,
            xyb_encoded,
            colour_encoding,
            tone_mapping,
            extensions
        })
    }
}

#[derive(Debug)]
struct JxlFrame {

}

impl JxlFrame {
    pub fn read(bitstream: &mut BitStream) -> Option<Self> {
        Some(JxlFrame{})
    }
}

pub fn decode_jxl(input_file: JxlFile) -> Option<PixelArray<u32>> {
    let mut jxl_data = BitStream::new(&input_file.get_image_data());
    assert_eq!(jxl_data.read_u16(16).unwrap(),0x0aff,"Invalid JXL");
    let image_size = JxlImageSize::read(&mut jxl_data).expect("Not enough data to read image size!");
    let image_metadata = JxlImageMetadata::read(&mut jxl_data).expect("Not enough data to read image size!");
    println!("Image dimensions: {:?}",image_size);
    println!("Image metadata: {:?}",image_metadata);
    let preview_frame = if image_metadata.preview_size != None {
        Some(JxlFrame::read(&mut jxl_data).unwrap())
    } else { None };
    let mut frames: Vec<JxlFrame> = Vec::new();
    loop {
        let next_frame = JxlFrame::read(&mut jxl_data).unwrap();
        frames.push(next_frame);
        break;
    }
    None
}