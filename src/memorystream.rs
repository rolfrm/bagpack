use std::io::{Write,Result, Read};

pub struct MemoryStream {
    pub buffer: Box<Vec<u8>>,
    offset: usize
}

impl MemoryStream{
    pub fn new() -> MemoryStream {
        MemoryStream { buffer: Box::new(Vec::new()), offset: 0 }
    }
    pub fn rewind(&mut self){
        self.offset = 0;
    }
    pub fn get_buffer(&self) -> Vec<u8>{
        return self.buffer.to_vec();
    }
}

impl Write for MemoryStream{
    fn write(&mut self, buf: &[u8]) -> Result<usize>{
        self.buffer.write(buf).unwrap();
        self.offset += buf.len();
        return Ok(buf.len());
    }
    fn flush(&mut self) -> Result<()>{
        return Ok(());
    }
}

impl Read for MemoryStream{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>{
        let copy_end = self.offset + buf.len();
        let copy_maxed = std::cmp::min(copy_end, self.buffer.len());
        for i in 0..copy_maxed {
            buf[i] = self.buffer[i + self.offset];
        }
        //buf.copy_from_slice(&self.buffer[self.offset..copy_maxed]);
        let s = copy_maxed - self.offset;
        self.offset = copy_maxed;
        return Ok(s);
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_memorystream() {
        let mut mem = Box::new(MemoryStream::new());
        let mut wrt = std::io::BufWriter::new(mem);
        
        let bytes = "hej".as_bytes();
        wrt.write(bytes).unwrap();
        wrt.flush().unwrap();
        
        if let Ok(x) = wrt.into_inner() {
            let v = x.get_buffer();
            
            assert_eq!(3, v.len());
            for  i in 0..3 {
                assert_eq!(bytes[i], v[i]);
            }
        }
    }
}