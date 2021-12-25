use byteorder::WriteBytesExt;
use log::debug;
use minecraft_varint::{VarIntRead, VarIntWrite};
use std::{io::Read, io::{Write, Cursor}};

use crate::varuint_size;

pub trait Packet<S: Read + Write> {
    fn read_with_len(r: &mut S, id: u32, len: u32) -> Option<Self> where Self: Sized {
        unimplemented!()
    }
    fn read(_r: &mut S) -> Option<Self>
    where
        Self: Sized,
    {
        unimplemented!()
    }
    
    fn write_impl(&self) -> Vec<u8> {
        unimplemented!()
    }
    fn write(&self, w: &mut S) -> Option<usize> {
        let v = self.write_impl();
        let lenlen = w.write_var_u32((v.len()).try_into().unwrap()).ok()?;
        let write = w.write(&v).unwrap();

        let mut c = Cursor::new(v.clone());
        let id = c.read_var_u32().unwrap();
        dbg!(id, v.len());
        debug!("-> packet {:x} with length {}.", id, lenlen + write);

        Some(write + lenlen)
    }
    fn get_id() -> u32;
}
pub trait ReadMcString {
    fn read_mc_string(&mut self) -> String;
}
impl<T> ReadMcString for T
where
    T: Read,
{
    fn read_mc_string(&mut self) -> String {
        let expected_string_length = self.read_var_u32().unwrap();
        // dbg!(expected_string_length);
        let mut buf = vec![0u8; expected_string_length.try_into().unwrap()];
        self.read_exact(&mut buf).unwrap();
        String::from_utf8(buf).unwrap()
    }
}
pub trait WriteMcString {
    fn write_mc_string(&mut self, s: String);
}
impl<T> WriteMcString for T
where
    T: Write,
{
    fn write_mc_string(&mut self, s: String) {
        self.write_var_u32((s.len()).try_into().unwrap()).unwrap();
        let chars = s.bytes();
        for ch in chars {
            self.write_u8(ch as u8).unwrap();
        }
    }
}
