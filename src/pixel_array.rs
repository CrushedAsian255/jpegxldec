#[derive(Debug)]
pub struct PixelArray<T> {
    width: u32,
    height: u32,
    channels: u16,
    buffer: Vec<T>
}

impl<T> PixelArray<T> {
    pub fn new(width: u32, height: u32, channels: u16) -> PixelArray<T> {
        Self {
            width,
            height,
            channels,
            buffer: Vec::with_capacity((width*height*(channels as u32)).try_into().unwrap())
        }
    }
}