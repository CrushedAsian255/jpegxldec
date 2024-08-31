mod jxl_file;
mod bit_reader;
mod jxl_decoder;
mod pixel_array;

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
    assert_eq!(jxl_file.print_box_list(),Some(()));
    let output_pixels = jxl_decoder::decode_jxl(jxl_file);
}