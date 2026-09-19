use std::{
    io::{Cursor, Write},
    sync::Arc,
};

use tcpserver::{IClient, client};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    channel::ChannelWeakHandle, gateway::{commands::EventId, events::IEventData, stateful::StatefulXor}, room::{MusicId, RoomWeakHandle}, session::Session, user::User,
};

const MAX_PACKET_SIZE: usize = 1024 * 8; // 8 KB
const INVALID_USER_ID: u64 = u64::MAX;

#[allow(dead_code)]
pub struct Client {
    pub id: u64,
    pub run: std::sync::Arc<std::sync::atomic::AtomicBool>,
    pub socket: tokio::net::TcpStream,
    pub buffer: [u8; MAX_PACKET_SIZE],
    pub data: Vec<u8>,
    pub token: tokio_util::sync::CancellationToken,

    // Server data:
    pub xor: StatefulXor,
    pub sender: Option<UnboundedSender<(EventId, Arc<dyn IEventData>)>>,
    pub client_list: Vec<MusicId>,
    pub user: Option<User>,

    pub queue: Vec<Vec<u8>>,
    pub queue_begin: bool,

    pub session: Option<Session>,
    pub channel_handle: Option<ChannelWeakHandle>,
    pub room_handle: Option<RoomWeakHandle>,
    pub version_checked: bool,
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

        // log::info!(
        //     "Sending packet to client {}: ID = {:#06X}, Length = {}",
        //     self.id, id, length
        // );

        self.send(&data).await?;
        Ok(())
    }

    pub fn begin(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.queue_begin = true;
        // self.queue.clear();
        Ok(())
    }

    pub async fn end(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.queue_begin = false;

        // for packet in std::mem::take(&mut self.queue) {
        //     self.send(&packet).await?;
        // }

        Ok(())
    }

    pub fn set_channel_handle(&mut self, handle: ChannelWeakHandle) {
        self.channel_handle = Some(handle);
    }

    pub fn channel(&self) -> Option<(u64, &ChannelWeakHandle)> {
        let Some(handle) = self.channel_handle.as_ref() else {
            return None;
        };

        let Some(user) = self.user.as_ref() else {
            return Some((INVALID_USER_ID, handle));
        };

        Some((user.id, handle))
    }

    pub fn room(&self) -> Option<(u64, &RoomWeakHandle)> {
        let Some(handle) = self.room_handle.as_ref() else {
            return None;
        };

        let Some(user) = self.user.as_ref() else {
            return Some((INVALID_USER_ID, handle));
        };

        Some((user.id, handle))
    }

    pub fn user(&self) -> Option<&User> {
        self.user.as_ref()
    }

    pub fn session_mut(&mut self) -> Option<&mut Session> {
        self.session.as_mut()
    }
}

client!(Client, |socket, run, id, token| {
    let xor = StatefulXor::new();

    Client {
        id,
        run,
        socket,
        token,
        buffer: [0; 1024 * 8],
        data: Vec::new(),
        xor,
        sender: None,
        client_list: Vec::new(),
        user: None,
        queue: Vec::new(),
        queue_begin: false,
        session: None,
        channel_handle: None,
        room_handle: None,
        version_checked: false,
    }
});
