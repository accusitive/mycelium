use std::{
    io::{Cursor, Read, Write},
};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use minecraft_varint::{VarIntRead, VarIntWrite};
use nibbler::nibble::Nibble;

use crate::{
    packet::{Packet, ReadMcString, WriteMcString},
};

#[derive(Debug)]
pub struct JoinGame {
    pub entity_id: u32,
    pub is_hardcore: bool,
    pub gamemode: u8,
    pub dimension: i32,

    pub difficulty: u8,
    pub max_players: u8,
    pub level_type: String,
    pub reduced_debug_info: bool,
}
#[derive(Debug)]
pub struct ClientSettings {
    pub locale: String,
    pub view_distance: i8,
    pub chat_mode: u32,
    pub chat_colors: bool,
    pub displayed_skin_parts: u8,
    /// 0: left, 1: right
    pub main_hand: u32,
    pub use_text_filtering: bool,
    pub allow_server_listing: bool,
}
#[derive(Debug)]
pub struct HeldItemChange {
    pub slot: u8,
}
#[derive(Debug)]
pub struct PluginMessageS {
    pub channel: String,
    pub data: Vec<u8>,
}
#[derive(Debug)]
pub struct PlayerPosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub on_ground: bool,
}
#[derive(Debug)]
pub struct ServerBoundPlayerPositionAndRotation {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}
#[derive(Debug)]
pub struct ClientBoundPlayerPositionAndRotation {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub flags: i8,
    // pub on_ground: bool,
    pub teleport_id: u32,
    pub dismount_veicle: bool,
}
#[derive(Debug)]

pub struct Palette {
    pub palette_length: u32,
    pub palette: Vec<u32>,
}
#[derive(Debug)]
pub struct ChunkSection {
    pub bits_per_block: u8,
    pub palette: Palette,
    pub data_array_length: u32,
    pub data_array: Vec<i64>,
    pub block_light: Vec<Nibble>,
    pub sky_light: Option<Vec<Nibble>>,
}
#[derive(Debug)]
pub struct ChunkColumn {
    pub sections: Vec<ChunkSection>,
    pub biomes: Option<Vec<u8>>,
}

#[derive(Debug)]
pub struct ChunkData {
    pub chunk_x: i32,
    pub chunk_z: i32,
    // /// always true
    // pub ground_up_continuous: bool,
    // pub primary_bit_mask: u32,

    // // Size and data
    // pub chunk_sections: Vec<ChunkColumn>,
    // // Number of block entities, and block entities
    // pub block_entities: Vec<nbt::Blob>,
}
#[derive(Debug)]
pub struct ClientBoundKeepAlive(pub i64);
#[derive(Debug)]
pub struct ServerBoundKeepAlive(pub  i64);

#[derive(Debug, Clone, Copy)]
pub enum ChatPosition {
    Chat = 0,
    System = 1,
    GameInfo = 2
}
#[derive(Debug)]
pub struct ClientBoundChat(pub String, pub ChatPosition);


impl<S: Read + Write> Packet<S> for JoinGame {
    fn write_impl(&self) -> Vec<u8> {
        let v = vec![];
        let mut c = Cursor::new(v);
        c.write_var_u32(<Self as Packet<S>>::get_id()).unwrap();
        c.write_u32::<BigEndian>(self.entity_id).unwrap();
        // c.write_u8(self.is_hardcore as u8).unwrap();
        c.write_u8(self.gamemode).unwrap();
        c.write_i32::<BigEndian>(self.dimension).unwrap();
        c.write_u8(self.difficulty).unwrap();
        c.write_u8(self.max_players).unwrap();
        c.write_mc_string(self.level_type.clone());
        c.write_u8(self.reduced_debug_info as u8).unwrap();
        c.into_inner()
    }
    fn get_id() -> u32 {
        0x23
    }
}
impl<S: Read + Write> Packet<S> for ClientSettings {
    fn read(r: &mut S) -> Option<Self>
    where
        Self: Sized,
    {
        let locale = r.read_mc_string();
        let view_distance = r.read_i8().ok()?;
        let chat_mode = r.read_var_u32().ok()?;
        let chat_colors = r.read_i8().ok()? == 0x01;
        let displayed_skin_parts = r.read_u8().ok()?;
        let main_hand = r.read_var_u32().ok()?;
        let use_text_filtering = r.read_i8().ok()? == 0x01;
        let allow_server_listing = r.read_i8().ok()? == 0x01;

        Some(Self {
            locale,
            view_distance,
            chat_mode,
            allow_server_listing,
            chat_colors,
            displayed_skin_parts,
            main_hand,
            use_text_filtering,
        })
    }
    fn get_id() -> u32 {
        0x05
    }
}
impl<S: Read + Write> Packet<S> for HeldItemChange {
    fn write_impl(&self) -> Vec<u8> {
        let v = vec![];
        let mut c = Cursor::new(v);
        c.write_var_u32(<Self as Packet<S>>::get_id()).unwrap();
        c.write_u8(self.slot).unwrap();

        c.into_inner()
    }
    fn get_id() -> u32 {
        0x3a
    }
}
impl<S: Read + Write> Packet<S> for PluginMessageS {
    fn read_with_len(r: &mut S, _id: u32, _len: u32) -> Option<Self>
    where
        Self: Sized,
    {
        let channel = r.read_mc_string();
        // let id_len = varuint_size(id);
        // dbg!(id_len);
        // let usize_len: usize = len.try_into().unwrap();
        // let len_of_data = usize_len - channel.len() - id_len - 1;
        let len_of_data = r.read_var_u32().ok()?.try_into().unwrap();
        let mut buf = vec![0; len_of_data];
        r.read_exact(&mut buf).unwrap();

        dbg!(&String::from_utf8(buf.clone()));
        Some(Self {
            channel: channel,
            data: buf,
        })
    }
    fn get_id() -> u32 {
        0x0a
    }
}
impl<S: Read + Write> Packet<S> for PlayerPosition {
    fn read(r: &mut S) -> Option<Self>
    where
        Self: Sized,
    {
        let x = r.read_f64::<BigEndian>().ok()?;
        let y = r.read_f64::<BigEndian>().ok()?;
        let z = r.read_f64::<BigEndian>().ok()?;
        let on_ground = r.read_i8().ok()? == 1;
        Some(Self { x, y, z, on_ground })
    }
    fn get_id() -> u32 {
        0x11
    }
}
impl<S: Read + Write> Packet<S> for ServerBoundPlayerPositionAndRotation {
    fn read(r: &mut S) -> Option<Self>
    where
        Self: Sized,
    {
        let x = r.read_f64::<BigEndian>().ok()?;
        let y = r.read_f64::<BigEndian>().ok()?;
        let z = r.read_f64::<BigEndian>().ok()?;
        let yaw = r.read_f32::<BigEndian>().ok()?;
        let pitch = r.read_f32::<BigEndian>().ok()?;

        let on_ground = r.read_i8().ok()? == 1;
        Some(Self {
            x,
            y,
            z,
            on_ground,
            yaw,
            pitch,
        })
    }
    fn get_id() -> u32 {
        0x11
    }
}
impl<S: Read + Write> Packet<S> for ChunkData {
    fn write_impl(&self) -> Vec<u8> {
        // let chunk_height = 256;
        // let section_height = 16;

        let v = vec![];
        let mut c = Cursor::new(v);
        c.write_var_u32(0x20).unwrap();

        c.write_i32::<BigEndian>(self.chunk_x).unwrap();
        c.write_i32::<BigEndian>(self.chunk_z).unwrap();
        c.write_u8(1).unwrap(); // true  for full
                                // let mask = u32::MAX;
        let mask = u32::MAX;
        c.write_var_u32(mask).unwrap();
        c.write_var_u32(0).unwrap();
        c.write_var_u32(0).unwrap();

        // let mut mask = 0;
        // let column_buffer = vec![];todo!()
        // for section in &self.chunk_sections {
        //     // mask |= (1 << );
        // }

        c.into_inner()
    }
    fn get_id() -> u32 {
        0x20
    }
}
impl<S: Read + Write> Packet<S> for ClientBoundPlayerPositionAndRotation {
    fn write_impl(&self) -> Vec<u8> {
        let v = vec![];
        let mut c = Cursor::new(v);
        c.write_var_u32(0x2f).unwrap();
        c.write_f64::<BigEndian>(self.x).unwrap();
        c.write_f64::<BigEndian>(self.y).unwrap();
        c.write_f64::<BigEndian>(self.z).unwrap();
        c.write_f32::<BigEndian>(self.yaw).unwrap();
        c.write_f32::<BigEndian>(self.pitch).unwrap();
        c.write_i8(self.flags).unwrap();
        c.write_var_u32(self.teleport_id).unwrap();
        // c.write_i8(self.dismount_veicle as i8).unwrap();
        dbg!(&c);
        c.into_inner()
    }
    fn get_id() -> u32 {
        0x23
    }
}
impl<S: Read + Write> Packet<S> for ClientBoundKeepAlive {
    fn write_impl(&self) -> Vec<u8> {
        let v = vec![];
        let mut c = Cursor::new(v);
        c.write_var_u32(<Self as Packet<S>>::get_id()).unwrap();

        c.write_i64::<BigEndian>(self.0).unwrap();
        c.into_inner()
    }
    fn get_id() -> u32 {
        0x1f
    }
}
impl<S: Read + Write> Packet<S> for ServerBoundKeepAlive {
    fn read(r: &mut S) -> Option<Self>
    where
        Self: Sized,
    {
        Some(Self(r.read_i64::<BigEndian>().unwrap()))
    }
    fn get_id() -> u32 {
        0x0b
    }
}
impl ClientBoundKeepAlive {
    pub fn new() -> Self {
        Self(
            // std::time::SystemTime::now()
            //     .duration_since(SystemTime::UNIX_EPOCH)
            //     .unwrap()
            //     .as_millis()
            //     .try_into()
            //     .unwrap(),
            420
        )
    }
}
impl<S: Read + Write> Packet<S> for ClientBoundChat {
    fn write_impl(&self) -> Vec<u8> {
        let mut v = vec![];
        v.write_var_u32(<Self as Packet<S>>::get_id()).unwrap();
        v.write_mc_string(self.0.clone());
        v.write_u8(self.1 as u8).unwrap();
        v
    }
    fn get_id() -> u32 {
        0x0f
    }
}