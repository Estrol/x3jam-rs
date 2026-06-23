pub mod connection;
pub mod game;
pub mod gateway;
pub mod listroom;
pub mod planet;
pub mod room;
pub mod shop;

use std::io::{Cursor, Read};

use byteorder_lite::{LittleEndian, ReadBytesExt};
use encoder::StructReadExt as _;
use tcpserver::IClient;

use crate::gateway::{cbc_decrypt, commands::RequestId};

use super::Client;

pub fn parse_request<T: encoder::StructDecodeImpl>(data: &[u8]) -> std::io::Result<T> {
    let mut cursor = Cursor::new(data);
    let mut request = std::mem::MaybeUninit::<T>::uninit();
    cursor.read_uninit(&mut request)?;
    Ok(unsafe { request.assume_init() })
}

#[derive(Debug)]
pub struct Packet {
    pub id: RequestId,
    pub body: Vec<u8>,
}

async fn make_packets(client: &mut Client) -> std::io::Result<Vec<Packet>> {
    let mut packets = Vec::new();
    let data = client.data().to_vec();
    let mut cursor = Cursor::new(&data);

    while cursor.position() < data.len() as u64 {
        let length = cursor.read_u16::<LittleEndian>()?;
        if length > data.len() as u16 {
            break; // Invalid length, stop processing
        }

        cursor.set_position(cursor.position() - 2);

        let mut packet_data = vec![0u8; length as usize];
        cursor.read_exact(&mut packet_data)?;

        let mut xored_data = packet_data[2..].to_vec();
        client.xor.process(&mut xored_data);

        let mut xored_cursor = Cursor::new(&xored_data);
        let mut password = [0u8; 16];
        let mut ex_end_block = [0u8; 8];

        xored_cursor.read_exact(&mut password)?;
        xored_cursor.read_exact(&mut ex_end_block)?;

        let body_size = xored_cursor.read_u32::<LittleEndian>()? as usize;
        let ex_body_with_size_pad = xored_cursor.read_u32::<LittleEndian>()? as usize;

        let body_size_with_pad = xored_data.len() - xored_cursor.position() as usize;
        if body_size_with_pad != ex_body_with_size_pad || body_size > body_size_with_pad {
            println!(
                "[Error] Body size mismatch: body_size={}, body_size_with_pad={}, ex_body_with_size_pad={}",
                body_size, body_size_with_pad, ex_body_with_size_pad
            );
            break; // Invalid body size, stop processing
        }

        let end_diff = 32 + body_size_with_pad - ex_end_block.len();

        let end_block = xored_data[end_diff..end_diff + ex_end_block.len()].to_vec();

        if ex_end_block != *end_block {
            println!(
                "[Error] End block mismatch: expected={:02X?}, actual={:02X?}",
                ex_end_block, end_block
            );
            break; // End block mismatch, stop processing
        }

        let payload_size = body_size - 2;
        let Some(des_key) = cbc_decrypt::Md5AndDes::derive_key(&password, Some(16)) else {
            println!("[Error] Failed to derive DES key from password");
            break; // Invalid password length, stop processing
        };

        let mut encypted_data = vec![0u8; body_size_with_pad];
        xored_cursor.read_exact(&mut encypted_data)?;

        let decrypted_data =
            match cbc_decrypt::Md5AndDes::decrypt_with_key(&encypted_data, &des_key) {
                Ok(data) => data,
                Err(_) => break, // Decryption failed, stop processing
            };

        let mut data_cursor = Cursor::new(&decrypted_data);
        let packet_id = RequestId::from_bytes(&mut data_cursor);
        let mut data = vec![0u8; payload_size];

        if payload_size > 0 {
            data_cursor.read_exact(&mut data)?;
        }

        packets.push(Packet {
            id: packet_id,
            body: data,
        });
    }

    Ok(packets)
}

pub async fn handle_request(client: &mut super::Client) {
    match make_packets(client).await {
        Ok(mut packets) => {
            if packets.len() > 0 && client.begin().is_ok() {
                for packet in packets.iter_mut() {
                    println!(
                        "Received packet: ID={:?} for client={:?}",
                        packet.id, client.id
                    );

                    match get_route(packet.id) {
                        Some(route) => (route.handler)(client, packet).await,
                        _ => println!("Received unhandled packet: ID={:?}", packet.id),
                    }
                }

                client.end().await.expect("Failed to end packet batch");
            }
        }
        Err(e) => eprintln!("Failed to process packets: {}", e),
    }
}

pub type Handler =
    for<'a> fn(
        &'a mut Client,
        &'a mut Packet,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>>;

pub struct Route {
    id: RequestId,
    handler: Handler,
}

inventory::collect!(Route);

pub fn get_route(id: RequestId) -> Option<&'static Route> {
    for route in inventory::iter::<Route> {
        if route.id == id {
            return Some(route);
        }
    }

    None
}
