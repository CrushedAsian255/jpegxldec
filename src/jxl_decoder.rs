use crate::bit_reader::{BitStream,CapU32Distributions};
use crate::jxl_file::JxlFile;

pub fn decode_jxl(input_file: JxlFile) {
    let mut jxl_data = BitStream::new(&input_file.get_image_data());
    assert_eq!(jxl_data.read_u16(16).unwrap(),0x0aff,"Invalid JXL");
}