mod jxl_file;
mod bit_reader;
mod jxl_image;
mod pixel_array;
mod jxl_frame;
mod decode_jxl;
mod decode_frame;
mod common;

use std::env;

#[allow(unused_mut,unused_variables)]
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Error: no input given");
        return;
    }
    println!("Opening file: {}",args[1]);
    let mut file = match std::fs::File::open::<&str>(args[1].as_ref()) {
        Ok(file) => file,
        Err(error) => {
            println!("Error reading file: {}",error);
            return;
        }
    };
    #[allow(unused_variables)]
    let jxl_file = jxl_file::JxlFile::read(file).unwrap();
    let output_pixels = decode_jxl::decode_jxl(jxl_file);
}