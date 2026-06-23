use std::{
    io::{Cursor, Write},
    sync::{Arc, Weak},
};

use tokio::sync::Mutex;
use tcpserver::{IClient, client};

use crate::gateway::{
    channel::Channel,
    commands::EventId,
    events::IEventData,
    room::{MusicId, Room},
    stateful::StatefulXor,
    user::User,
};

const MAX_PACKET_SIZE: usize = 1024 * 8; // 8 KB

#[allow(dead_code)]
pub struct Client {
    pub id: u64,
    pub run: std::sync::Arc<std::sync::atomic::AtomicBool>,
    pub socket: tokio::net::TcpStream,
    pub buffer: [u8; MAX_PACKET_SIZE],
    pub data: Vec<u8>,

    // Server data:
    pub xor: StatefulXor,
    pub login_xor_char: u8,
    pub sender: Option<Arc<tokio::sync::mpsc::UnboundedSender<(EventId, Arc<dyn IEventData>)>>>,
    pub client_list: Vec<MusicId>,
    pub channel: Option<Weak<Mutex<Channel>>>,
    pub room: Option<Weak<Mutex<Room>>>,
    pub user: Option<User>,

    pub queue: Vec<Vec<u8>>,
    pub queue_begin: bool,

    pub session_entered: bool,
}

impl Client {
    pub async fn send_packet<ID, DATA>(
        &mut self,
        id: ID,
        body: &DATA,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        ID: Into<u16>,
        DATA: encoder::StructEncodeImpl,
    {
        use encoder::StructWriteExt as _;

        let mut data = Cursor::new(Vec::new());
        data.write_all(&[0u8; 2])?; // Placeholder for length

        let id = id.into();

        data.write_struct(&id)?;
        data.write_struct(body)?;

        let length = data.get_ref().len() as u16;
        data.set_position(0);
        data.write_all(&length.to_le_bytes())?;

        let data = data.into_inner();

        println!(
            "Sending packet to client {}: ID = {:#06X}, Length = {}",
            self.id, id, length
        );

        self.queue.push(data);
        Ok(())
    }

    pub fn begin(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.queue_begin = true;
        self.queue.clear();
        Ok(())
    }

    pub async fn end(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.queue_begin = false;

        // const MAX_SEND_AT_ONE_TIME: usize = MAX_PACKET_SIZE;
        // let mut combined_data = Vec::new();

        // let packets = std::mem::take(&mut self.queue);
        // for packet in packets {
        //     if !combined_data.is_empty()
        //         && combined_data.len() + packet.len() > MAX_SEND_AT_ONE_TIME
        //     {
        //         self.send(&combined_data).await?;
        //         combined_data.clear();
        //     }

        //     if packet.len() > MAX_SEND_AT_ONE_TIME {
        //         self.send(&packet).await?;
        //     } else {
        //         combined_data.extend_from_slice(&packet);
        //     }
        // }

        // if !combined_data.is_empty() {
        //     self.send(&combined_data).await?;
        // }

        for packet in std::mem::take(&mut self.queue) {
            self.send(&packet).await?;
        }

        Ok(())
    }

    pub fn channel(&self) -> Option<Arc<Mutex<Channel>>> {
        self.channel.as_ref()?.upgrade()
    }

    pub fn room(&self) -> Option<Arc<Mutex<Room>>> {
        self.room.as_ref()?.upgrade()
    }

    pub fn user(&mut self) -> Option<&mut User> {
        self.user.as_mut()
    }

    pub fn clear(&mut self) {
        self.channel = None;
        self.room = None;
        self.user = None;
    }
}

client!(Client, |socket, run, id| {
    let xor = StatefulXor::new();

    Client {
        id,
        run,
        socket,
        buffer: [0; 1024 * 8],
        data: Vec::new(),
        xor,
        login_xor_char: 0,
        sender: None,
        client_list: Vec::new(),
        channel: None,
        room: None,
        user: None,
        queue: Vec::new(),
        queue_begin: false,
        session_entered: false,
    }
});
