#[derive(Debug,PartialEq)]
pub struct ImageSize {
    pub width: u32,
    pub height: u32
}

pub fn unpack_signed(val: u32) -> i32 {
    if val & 1 == 0 { (-(val as i64 + 1)/2) as i32 } else { (val / 2) as i32 }
}