use crate::bit_reader::BitStream;
use crate::jxl_image::JxlImageMetadata;
use crate::jxl_file::JxlFile;
use crate::jxl_frame::JxlFrame;
use crate::common::ImageSize;

pub fn decode_jxl(input_file: JxlFile) {
    let mut jxl_data = BitStream::new(&input_file.get_image_data());
    assert_eq!(jxl_data.read_u16(16).unwrap(),0x0aff,"Invalid JXL");
    let image_size = ImageSize::read(&mut jxl_data).expect("Not enough data to read image size!");
    let image_metadata = JxlImageMetadata::read(&mut jxl_data).expect("Not enough data to read image size!");
    println!("Image dimensions: {:?}",image_size);
    println!("Image metadata: {:?}",image_metadata);
    let _preview_frame = if image_metadata.preview_size != None {
        Some(JxlFrame::read(&mut jxl_data,&image_metadata).unwrap())
    } else { None };
    let mut frames: Vec<JxlFrame> = Vec::new();
    loop {
        let next_frame = JxlFrame::read(&mut jxl_data,&image_metadata).unwrap();
        frames.push(next_frame);
        break;
    }
    println!("{:?}",frames);
}