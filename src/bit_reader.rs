// TODO: Write a more optimised BitStream class that doesn't just read each bit one-by-one as that is stupid
#![allow(dead_code)]

pub struct BitStream {
    data: Vec<u8>,
    ptr: usize,
    tail: u8,
    tail_len: u8
}

#[derive(Copy, Clone, Debug)]
pub enum QuadDistributions {
    RawValue(u32),
    BitCount(u8),
    BitCountWithOffset(u8, u32)
}

impl BitStream {
    pub fn new<'a>(data: &[u8]) -> Self { Self { data: data.to_owned(), ptr: 0, tail: data[0], tail_len: 8 } }
}

impl BitStream {
    pub fn is_empty(&self) -> bool {
        self.tail_len == 0 && self.ptr == self.data.len()
    } 
    pub fn pad_zero(&mut self) -> Option<()> {
        if self.is_empty() { None }
        else if self.tail != 0 { None }
        else { Some(()) }
    }
    fn read_bit(&mut self) -> Option<u8> {
        if self.tail_len == 0 {
            if self.ptr == (self.data.len() - 1) {
                return None;
            } else {
                self.ptr += 1;
                self.tail = self.data[self.ptr];
                self.tail_len = 8;
            }
        }
        let out = self.tail & 0x1;
        self.tail >>= 1;
        self.tail_len -= 1;
        return Some(out);
    }
    pub fn read_bool(&mut self) -> Option<bool> {
        let out = self.read_bit()?;
        #[cfg(feature = "bit_read_debug_prints")]
        println!("Read bool: {}",out);
        Some(out != 0)
    }
    pub fn read_u8(&mut self, bits: u8) -> Option<u8> {
        let mut out: u8 = 0;
        for i in 0..bits {
            out |= self.read_bit()? << i;
        }
        #[cfg(feature = "bit_read_debug_prints")]
        println!("Read u8({}): {}",bits,out);
        Some(out)
    }
    pub fn read_u16(&mut self, bits: u8) -> Option<u16> {
        let mut out: u16 = 0;
        for i in 0..bits {
            out |= (self.read_bit()? as u16) << i;
        }
        #[cfg(feature = "bit_read_debug_prints")]
        println!("Read u16({}): {}",bits,out);
        Some(out)
    }
    pub fn read_u32(&mut self, bits: u8) -> Option<u32> {
        let mut out: u32 = 0;
        for i in 0..bits {
            out |= (self.read_bit()? as u32) << i;
        }
        #[cfg(feature = "bit_read_debug_prints")]
        println!("Read u32({}): {}",bits,out);
        Some(out)
    }
    pub fn read_u64(&mut self, bits: u8) -> Option<u64> {
        let mut out: u64 = 0;
        for i in 0..bits {
            out |= (self.read_bit()? as u64) << i;
        }
        #[cfg(feature = "bit_read_debug_prints")]
        println!("Read u64({}): {}",bits,out);
        Some(out)
    }
    pub fn read_quad_u32(&mut self, d0: QuadDistributions, d1: QuadDistributions, d2: QuadDistributions, d3: QuadDistributions) -> Option<u32> {
        let distribution = self.read_u8(2)?;
        let selected_distribution = [d0,d1,d2,d3][distribution as usize];
        
        let out = match selected_distribution {
            QuadDistributions::RawValue(n) => n,
            QuadDistributions::BitCount(n) => self.read_u32(n)?,
            QuadDistributions::BitCountWithOffset(n, o) => self.read_u32(n)?.wrapping_add(o)
        };

        #[cfg(feature = "bit_read_debug_prints")]
        println!("Read U32({:?},{:?},{:?},{:?}): {}:{}",d0,d1,d2,d3,distribution,out);
        
        Some(out)
    }
    pub fn read_var_u64(&mut self) -> Option<u64> {
        match self.read_u8(2)? {
            0 => Some(0),
            1 => Some(self.read_u8(4)? as u64 + 1),
            2 => Some(self.read_u8(8)? as u64 + 17),
            3 => {
                todo!("Not implemented: U64.3");
            }
            _ => unreachable!()
        }
    }
}

#[cfg(test)]
mod bit_stream_tests {
    use crate::bit_reader::BitStream;
    
    #[test]
    fn read_bits() {
        let mut stream = BitStream::new(Vec::from([0b0011_0111,0b1001_0110,0b1111_0010]).as_slice());
        let mut str: String = String::new();
        loop {
            str.push_str(&format!("{}",match stream.read_bit() {Some(x)=>x, None=>break}));
        }
        assert_eq!("111011000110100101001111",str);
    }

    #[test]
    fn read_bytes_aligned() {
        let mut stream = BitStream::new(Vec::from([0b0011_0111,0b1001_0110,0b1111_0010]).as_slice());
        assert_eq!(stream.read_u8(8),Some(0b0011_0111));
        assert_eq!(stream.read_u8(8),Some(0b1001_0110));
        assert_eq!(stream.read_u8(8),Some(0b1111_0010));        
    }

    #[test]
    fn read_bytes_misaligned() {
        let mut stream = BitStream::new(Vec::from([0b0011_0111,0b1001_0110,0b1111_0010]).as_slice());
        assert_eq!(stream.read_u8(3),Some(0b111));
        assert_eq!(stream.read_u8(8),Some(0b110_0011_0));
        assert_eq!(stream.read_u8(8),Some(0b010_1001_0));       
    }

    #[test]
    fn read_a_lot() {
        let mut stream = BitStream::new(Vec::from([
            0b0011_0111,0b1001_0110,0b1111_0010,0b1101_1011,
            0b0101_0101,0b0000_1111,0b1010_1010,0b1111_0000,0b0101_0101,0b0000_1111,0b1010_1010,0b1111_0000,
            0b1011_0111,0b1111_1100
        ]).as_slice());
        assert_eq!(stream.read_u32(32),Some(0b1101_1011_1111_0010_1001_0110_0011_0111));
        assert_eq!(stream.read_u64(64),Some(0b1111_0000_1010_1010_0000_1111_0101_0101_1111_0000_1010_1010_0000_1111_0101_0101));   
        assert_eq!(stream.read_u16(16),Some(0b1111_1100_1011_0111));
    }
}