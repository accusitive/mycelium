use std::io::{Cursor, Read, Write};

use byteorder::{BigEndian, WriteBytesExt};
use minecraft_varint::VarIntWrite;

use crate::packet::{Packet, ReadMcString, WriteMcString};

#[derive(Debug)]
pub struct LoginStart {
    pub username: String,
}
#[derive(Debug)]
pub struct LoginSuccess {
    pub uuid: u128,
    /// Whether the UUID should be sent along the network as a string
    pub as_string: bool,
    pub username: String,
}
impl<S: Read + Write> Packet<S> for LoginStart {
    fn get_id() -> u32 {
        0x00
    }
    fn read(r: &mut S) -> Option<Self>
    where
        Self: Sized,
    {
        Some(Self {
            username: r.read_mc_string(),
        })
    }
}
impl<S: Read + Write> Packet<S> for LoginSuccess {
    fn get_id() -> u32 {
        0x02
    }

    fn write_impl(&self) -> Vec<u8> {
        let v = vec![];
        let mut c = Cursor::new(v);
        c.write_var_u32(0x02).unwrap();

        if self.as_string {
            c.write_mc_string("00000000-0000-0000-0000-000000000000".to_string());
        } else {
            c.write_u128::<BigEndian>(self.uuid).unwrap();
        }
        c.write_mc_string(self.username.clone());

        c.into_inner()
    }
}
