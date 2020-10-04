extern crate crypto;
extern crate rand;

use std::io::prelude::*;
use rand::RngCore;
use crypto::{ aes, blockmodes };
use crypto::buffer::{ ReadBuffer, WriteBuffer, BufferResult };
use crypto::symmetriccipher::Encryptor;
use crate::memorystream::MemoryStream;
use crypto::buffer::{RefReadBuffer, RefWriteBuffer};

pub struct EncryptStream<W: Write>{
    inner: Option<W>,
    enc : Box<Encryptor>,
    write_buf: Vec<u8>,
    offset: usize
}

impl <W: Write> EncryptStream<W>{
    pub fn get_mut(&mut self) -> &mut W {
        self.inner.as_mut().unwrap()
    }

    pub fn finish(&mut self){
        let buf : Vec<u8> = Vec::new();
        self.write2(&buf, true);
    }

    fn write2(&mut self, buf: &[u8], eof: bool) -> std::io::Result<usize>{
        let mut rd = RefReadBuffer::new(buf);
        
        let mut ctn = true;
        //let wrt = ;
        while ctn {
               let mut wd = RefWriteBuffer::new(&mut self.write_buf);
               wd.pos = self.offset;
               let result = self.enc.encrypt(&mut rd, &mut wd, eof);
               self.offset = wd.pos;
               println!("Ok... {} {}", self.offset, buf.len());
               let mut rd2 = wd.take_read_buffer();
               let rem = rd2.take_remaining().to_vec();
               self.get_mut().write(rem.as_slice());
               if let Ok(r) = result {
                println!("Ok...");
                match r {
                BufferResult::BufferUnderflow => ctn = false,
                BufferResult::BufferOverflow => {}
                }
                
            }else{
                panic!("Oh no!");
            }
        }
        
        return Ok(buf.len());
    }
}

impl <W: Write> Write for EncryptStream<W>{

    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize>{
        return self.write2(buf, false);
    }
    fn flush(&mut self) -> std::io::Result<()>{
        return self.get_mut().flush();
    }
}


pub fn  new_aes_encrypt_stream<W: Write>(key : String, out: W) -> EncryptStream<W>{
    let mut key: [u8; 32] = [0; 32];
    let mut iv: [u8; 16] = [0; 16];
    let mut rng = rand::rngs::OsRng;
    rng.fill_bytes(&mut key);
    rng.fill_bytes(&mut iv);

    let mut encryptor = aes::cbc_encryptor(
        aes::KeySize::KeySize256,
        &key,
        &iv,
        blockmodes::PkcsPadding);
    
    return EncryptStream {
        inner: Option::from(out), 
        enc: encryptor, 
        write_buf: vec![0; 4096], offset: 0

    };
}




#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_aes_cryptostream() {
        let mut mem = MemoryStream::new();
        let mut enc = new_aes_encrypt_stream(String::from("key: String"), Box::new(&mut mem));
        enc.write("hej".as_bytes());
        enc.write("hej".as_bytes());
        enc.write("hej".as_bytes());
        enc.write("hej".as_bytes());
        enc.write("he2".as_bytes());
        //enc.write("hejhejhejhej".as_bytes());
        enc.finish();
        enc.flush();
        
        mem.rewind();
        let buf = mem.get_buffer();

        println!("{:?}", buf);
        
    }
}