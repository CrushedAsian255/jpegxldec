use crate::bit_reader::BitStream;
use crate::bit_reader::QuadDistributions::*;
use crate::jxl_file::JxlFile;
use crate::pixel_array::PixelArray;

#[derive(Debug)]
struct ImageSize {
    width: u32,
    height: u32
}

impl ImageSize {
    pub fn read(bitstream: &mut BitStream) -> Option<Self> {
        let div8 = bitstream.read_bool()?;
        let height = if div8 {
            (bitstream.read_u8(5)? + 1) as u32
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
                (bitstream.read_u8(5)? + 1) as u32
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
        Some(ImageSize { width, height })
    }
}

pub fn decode_jxl(input_file: JxlFile) -> Option<PixelArray<u32>> {
    let mut jxl_data = BitStream::new(&input_file.get_image_data());
    assert_eq!(jxl_data.read_u16(16).unwrap(),0x0aff,"Invalid JXL");
    let image_size = ImageSize::read(&mut jxl_data).expect("Not enough data to read image size!");
    
    println!("Image dimensions: {:?}",image_size);
    None
}