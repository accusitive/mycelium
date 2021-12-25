use std::io::{Cursor, Read, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use log::{info, warn};
use minecraft_varint::{VarIntRead, VarIntWrite};

use crate::{
    packet::{Packet, ReadMcString, WriteMcString},
    response_data::ResponseData,
};
type VarIntType = u32;
#[derive(Debug)]
pub struct Handshake {
    pub protocol: VarIntType,
    pub address: String,
    pub port: u16,
    pub next_state: VarIntType,
}
#[derive(Debug)]
pub struct Request {}
#[derive(Debug)]
pub struct Response {
    pub data: ResponseData,
}

#[derive(Debug)]
pub struct Ping {
    pub payload: u64,
}
#[derive(Debug)]
pub struct Pong {
    pub payload: u64,
}
impl<S: Read + Write> crate::packet::Packet<S> for Handshake {
    fn read(r: &mut S) -> Option<Self> {
        let len = r.read_var_u64().ok()?;
        if len == 0xfe {
            warn!("Client is connecting with legacy server ping.");
            return None;
        }
        let _id = r.read_var_u64().ok()?;

        let protocol = r.read_var_u32().ok()?;
        info!("Client connecting with protocol {}.", protocol);

        let address = r.read_mc_string();
        let port = r.read_u16::<BigEndian>().ok()?;
        let next_state = r.read_var_u32().ok()?;
        Some(Self {
            address,
            next_state,
            port,
            protocol,
        })
    }

    fn write_impl(&self) -> Vec<u8> {
        todo!()
    }

    fn get_id() -> u32 {
        0
    }
}
impl<S: Read + Write> Packet<S> for Request {
    fn read(r: &mut S) -> Option<Self>
    where
        Self: Sized,
    {
        // let _len = r.read_var_u64().ok()?;
        // let id = r.read_var_u64().ok()?;

        Some(Request {})
    }

    fn write_impl(&self) -> Vec<u8> {
        todo!()
    }

    fn get_id() -> u32 {
        0
    }
}
impl<S: Read + Write> Packet<S> for Response {
    fn read(r: &mut S) -> Option<Self>
    where
        Self: Sized,
    {
        todo!()
    }

    fn write_impl(&self) -> Vec<u8> {
        let buf = vec![];
        let mut c = Cursor::new(buf);
        c.write_var_u64(0x0).unwrap();

        let s = serde_json::to_string_pretty(&self.data).unwrap();
        c.write_mc_string(s);
        c.into_inner()
    }

    fn get_id() -> u32 {
        0
    }
}
impl<S: Read + Write> Packet<S> for Ping {
    fn read(r: &mut S) -> Option<Self>
    where
        Self: Sized,
    {
        let payload = r.read_u64::<BigEndian>().ok()?;
        Some(Ping { payload })
    }

    fn get_id() -> u32 {
        1
    }
}
impl<S: Read + Write> Packet<S> for Pong {
    fn write_impl(&self) -> Vec<u8> {
        let buf = vec![];
        let mut c = Cursor::new(buf);
        c.write_var_u64(0x1).unwrap();
        c.write_u64::<BigEndian>(self.payload).unwrap();
        let i = c.into_inner();
        i
    }

    fn get_id() -> u32 {
        1
    }
}
