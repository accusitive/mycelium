use std::cell::RefCell;
use std::io::{Cursor, Error, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

// use byteorder::{BigEndian, WriteBytesExt};
use log::{debug, error, info, warn};
use minecraft_varint::{VarIntRead, VarIntWrite};
use nbt::Map;

use crate::packets::handshake::*;
use crate::packets::login::{LoginStart, LoginSuccess};
use crate::packets::play::{
    ChunkData, ClientBoundChat, ClientBoundKeepAlive, ClientBoundPlayerPositionAndRotation,
    ClientSettings, HeldItemChange, JoinGame, PlayerPosition, PluginMessageS, ServerBoundKeepAlive,
    ServerBoundPlayerPositionAndRotation, ChatPosition,
};
use crate::{
    packet::Packet,
    response_data::{Description, Players, ResponseData, Sample, Version},
};
use std::collections::HashMap;
mod packet;
mod packets;
mod response_data;
#[derive(Debug)]
enum Message {
    PlayerJoined(String),
    ConnectionClosed,
}
fn main() {
    flexi_logger::Logger::try_with_str("debug")
        .unwrap()
        .start()
        .unwrap();
    info!("Started.");

    let listener = TcpListener::bind("127.0.0.1:8001").unwrap();
    let favicon = Arc::new(base64::encode(std::fs::read("./favicon.png").unwrap()));
    let clients: Arc<Mutex<HashMap<usize, (Receiver<Message>, RefCell<TcpStream>)>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let clients2 = clients.clone();

    std::thread::spawn(move || {
        for (index, client) in listener.incoming().enumerate() {
            let (tx, rx) = std::sync::mpsc::channel::<Message>();
            let favicon = favicon.clone();
            let clients = clients.clone();
            std::thread::spawn(move || {
                let stream = client.unwrap();
                // let i = {clients.lock().unwrap()}.len();
                // dbg!(index);
                clients
                    .lock()
                    .unwrap()
                    .insert(index, (rx, RefCell::new(stream.try_clone().unwrap())));

                let mut handler = ConnectionHandler {
                    favicon,
                    stream,
                    tx,
                };

                handler.handle_client();
                // dbg!(clients.lock().unwrap().remove(i+1));
                // println!("Done handling");
                // handle_client(client.unwrap(), favicon);
            });
        }
    });
    std::thread::spawn(move || {
        let mut is_running = true;
        let mut last_tick = SystemTime::now();
        let mut last_keep_alive = SystemTime::now();

        while is_running {
            let this_tick = SystemTime::now();

            if this_tick.duration_since(last_tick).unwrap().as_millis() > (1000) {
                last_tick = this_tick;
                let mut should_keep_alive = SystemTime::now()
                    .duration_since(last_keep_alive)
                    .unwrap()
                    .as_millis()
                    > 2500;
                let mut to_delete = Vec::new();
                {
                    let mutex_guard = clients2.lock().unwrap();
                    for (index, (rx, client)) in mutex_guard.iter() {
                        match rx.try_recv() {
                            Ok(Message::PlayerJoined(name)) => {
                                info!("Player `{}` joined!", name);
                                for x in mutex_guard.iter() {
                                    ClientBoundChat(
                                        format!("{{\"text\": \"+{} joined.\"}}", name),
                                        ChatPosition::Chat,
                                    ).write(&mut *x.1.1.borrow_mut());
                                }
                            }
                            Ok(Message::ConnectionClosed) => {
                                // mutex_guard.remove(index);
                                to_delete.push(*index);
                                // continue;
                                should_keep_alive = false;
                            }
                            _ => {}
                        }
                        // if client.peek(&mut []).is_err() {
                        //     clients2.lock().unwrap().remove(index);
                        // }
                        if should_keep_alive {
                            ClientBoundKeepAlive::new().write(&mut *client.borrow_mut());
                            last_keep_alive = SystemTime::now();
                        }
                    }
                }
                for entry in to_delete {
                    clients2.lock().unwrap().remove(&entry);
                    info!("Removed {}", entry);
                }
                println!("Ticking. {:?}", clients2.lock().unwrap());
            }
            // std::sync::mpsc::

            // std::thread::sleep_ms(200);
        }
    })
    .join()
    .unwrap();
}
/// Handler of just one player on its own thread
struct ConnectionHandler {
    stream: TcpStream,
    favicon: Arc<String>,
    tx: Sender<Message>,
}
impl ConnectionHandler {
    fn handle_client(&mut self) {
        info!("New client.");
        match Handshake::read(&mut self.stream) {
            Some(hs) => match hs.next_state {
                1 => self.handle_ping(&hs),
                2 => self.handle_play(&hs),
                _ => {
                    error!("Invalid next_state!")
                }
            },
            None => {
                warn!("Failed to read handshake. Killing client.");
                self.tx.send(Message::ConnectionClosed).unwrap();
            }
        }
    }
    fn handle_ping(&mut self, hs: &Handshake) {
        let _req: Request = read_packet(&mut self.stream).unwrap();

        let response = Response {
            data: ResponseData {
                version: Version {
                    name: "1.18.1".to_string(),
                    protocol: hs.protocol,
                },
                description: Description {
                    text: "§l§nMycelium Server 0.0".to_string(),
                },
                players: Players {
                    max: i32::MAX,
                    online: 10,
                    sample: vec![Sample {
                        id: "00000000-0000-0000-0000-000000000000".to_string(),
                        name: "aa".to_string(),
                    }],
                },
                favicon: Some(format!("data:image/png;base64,{}", self.favicon)),
            },
        };
        response.write(&mut self.stream).unwrap();
        let ping: Option<Ping> = read_packet(&mut self.stream);
        if let Some(p) = ping {
            Pong { payload: p.payload }.write(&mut self.stream);
            self.stream.shutdown(std::net::Shutdown::Both).unwrap();
            info!("Finished ping, exiting.");
            self.tx.send(Message::ConnectionClosed).unwrap();
            return;
        }
    }
    fn handle_play(&mut self, hs: &Handshake) {
        info!("Handling play.");
        // let root = quartz_nbt::snbt::parse(include_str!("../default.snbt")).unwrap();
        // h.insert("minecraft:dimension_type", nbt::Value::Compound());
        // let c = nbt::Value::Compound(h);
        let k: LoginStart = read_packet(&mut self.stream).unwrap();
        self.tx
            .send(Message::PlayerJoined(k.username.clone()))
            .unwrap();

        dbg!(&k);
        LoginSuccess {
            as_string: hs.protocol <= 572,
            username: k.username.clone(),
            uuid: 1293876,
        }
        .write(&mut self.stream);
        // std::thread::sleep(Duration::from_millis(200));

        JoinGame {
            difficulty: 0,
            dimension: 0,
            entity_id: 0,
            gamemode: 1,
            is_hardcore: false,
            level_type: "default".to_string(),
            max_players: 10,
            reduced_debug_info: false,
        }
        .write(&mut self.stream)
        .unwrap();
        // Main packet loop

        // HeldItemChange { slot: 0}
        //     .write(&mut self.stream)
        //     .unwrap();\
        // std::thread::sleep(Duration::from_millis(200));
        ClientBoundPlayerPositionAndRotation {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            flags: 0,
            dismount_veicle: false,
            pitch: 0.0,
            yaw: 0.0,
            teleport_id: 0,
        }
        .write(&mut self.stream);
        for x in 0..16 {
            for y in 0..16 {
                ChunkData {
                    chunk_x: x,
                    chunk_z: y,
                }
                .write(&mut self.stream)
                .unwrap();
            }
        }

        let mut expected_keepalive = 0;
        while self.stream.peek(&mut [0; 16]).unwrap() > 0 {
            let len = self.stream.read_var_u32().unwrap();
            if len == 0 {
                continue;
            }
            let id = self.stream.read_var_u32().unwrap();
            // let c = ClientBoundKeepAlive::new();
            // expected_keepalive = c.0;
            // dbg!(&c);
            // c.write(&mut self.stream);
            // debug!("read packet with id {:#x?} ({})", id, len);
            // let stone = include_bytes!("../Stone.bin");
            // {
            //     let v= vec![];
            //     let mut c = Cursor::new(v);
            //     c.write_all(stone).unwrap();
            // }

            // let try_into: usize = varuint_size(0x22).try_into().unwrap();
            // self.stream.write_var_u32((try_into + stone.len()).try_into().unwrap()).unwrap();
            // self.stream.write_var_u32(0x22).unwrap();
            // self.stream.write_all(stone).unwrap();
            match id {
                0x04 => {
                    let settings = ClientSettings::read(&mut self.stream).unwrap();
                    dbg!(settings);
                }
                0x09 => {
                    let p: PluginMessageS =
                        PluginMessageS::read_with_len(&mut self.stream, id, len).unwrap();
                    dbg!(p);
                }
                0x0d => {
                    let pos: PlayerPosition = PlayerPosition::read(&mut self.stream).unwrap();
                    // dbg!(pos);
                }
                0x0e => {
                    let posros: ServerBoundPlayerPositionAndRotation =
                        ServerBoundPlayerPositionAndRotation::read(&mut self.stream).unwrap();
                    // dbg!(posros);
                }
                0x0 => {
                    println!("TEleport confirm?");
                    let id = self.stream.read_var_u32().unwrap();
                    dbg!(id);
                }
                0xb => {
                    let keep_alive = ServerBoundKeepAlive::read(&mut self.stream).unwrap();
                    // assert_eq!(keep_alive.0, expected_keepalive);
                }
                _ => {
                    let x = (TryInto::<usize>::try_into(len).unwrap()) - varuint_size(len);
                    let mut buf = vec![0; x];
                    self.stream.read_exact(&mut buf).unwrap();
                    warn!(
                        "Packet with ID {:#X?} and length {} has been thrown away.",
                        id, len
                    );
                    // error!("Didn't recognize packet, breaking out!");
                    // break;
                }
            }
        }
        self.tx.send(Message::ConnectionClosed).unwrap();
        info!("Sent connection closed info.")
        //TODO: recipes
        //TODO: Tags
        //TODO: entity status
    }
}

pub fn varuint_size(i: u32) -> usize {
    let v = vec![];
    let mut cursor = Cursor::new(v);
    cursor.write_var_u32(i).unwrap();
    cursor.into_inner().len()
}
fn read_packet<S: Read + Write, P: Packet<S>>(s: &mut S) -> Option<P> {
    let len = s.read_var_u32().unwrap();
    let id = s.read_var_u32().unwrap();
    assert_eq!(id, P::get_id());
    debug!("<- Packet {:X} with len {}.", id, len);
    P::read(s)
}

// mod test {
//     use crate::test;

//     trait Version {
//         fn get_id(k: PacketKind) -> u32;
//     }
//     struct Mc1_18_1;
//     struct Mc1_17_1;

//     impl Version for Mc1_18_1 {
//         fn get_id(k: PacketKind) -> u32 {
//             match k {
//                 PacketKind::JoinGame => 0x1,
//                 PacketKind::LoginSuccess => 0x2,
//             }
//         }
//     }
//     impl Version for Mc1_17_1 {
//         fn get_id(k: PacketKind) -> u32 {
//             match k {
//                 PacketKind::JoinGame => 0x2,
//                 PacketKind::LoginSuccess => 0x3,
//             }
//         }
//     }
//     enum PacketKind {
//         JoinGame,
//         LoginSuccess,
//     }
//     trait Packet<V: Version> {
//         fn read();
//         fn write();
//         fn get_kind() -> PacketKind;
//         fn get_packet_id() -> u32 {
//             V::get_id(<Self as Packet<V>>::get_kind())
//         }
//     }
//     struct JoinGame;
//     impl<V: Version> Packet<V> for JoinGame {
//         fn read() {
//             todo!()
//         }

//         fn write() {
//             todo!()
//         }

//         fn get_kind() -> PacketKind {
//             PacketKind::JoinGame
//         }
//     }
//     #[test]
//     fn test_it() {
//         use Packet;
//         let j = JoinGame;
//         test::JoinGame::get_packet_id();

//     }
// }
