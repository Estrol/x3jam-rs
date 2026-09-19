use std::{
    any::TypeId,
    sync::{Arc, atomic::AtomicUsize},
};

use futures::FutureExt as _;
use tokio::{
    sync::{mpsc::UnboundedSender, oneshot},
    task::JoinHandle,
};

use crate::{
    gateway::{commands::EventId, events::IEventData, routes::myroom::InventoryEquipResponse},
    itemlist::ItemGender,
    room::{MusicId, RoomMode},
    user::{ItemId, User},
};

pub mod channel;
pub mod ojnlist;
pub mod user_repository;

#[derive(Debug)]
pub struct ChannelHandle {
    pub region: u32,
    pub id: u32,

    pub sender: tokio::sync::mpsc::UnboundedSender<ChannelRequest>,

    pub max_users: usize,
    pub counter: Arc<AtomicUsize>,
}

pub type Response = Box<dyn std::any::Any + Send + Sync + 'static>;

pub struct ChannelRequest {
    pub sender: Option<oneshot::Sender<Response>>,
    pub data: Option<ChannelCommand>,
}

impl ChannelRequest {
    pub fn send<DST: std::any::Any + Send + Sync + 'static>(mut self, response: DST) {
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(Box::new(response));
        }
    }
}

impl Drop for ChannelRequest {
    fn drop(&mut self) {
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(Box::new(()));
        }
    }
}

impl ChannelHandle {
    pub fn region(&self) -> u32 {
        self.region
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn max_users(&self) -> usize {
        self.max_users
    }

    pub fn current_users(&self) -> usize {
        self.counter.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub async fn send<DST: std::any::Any + Send>(
        &self,
        request: ChannelCommand,
    ) -> Result<DST, Box<dyn std::error::Error + Send + Sync>> {
        if TypeId::of::<()>() == TypeId::of::<DST>() {
            let channel_request = ChannelRequest {
                sender: None,
                data: Some(request),
            };

            self.sender
                .send(channel_request)
                .map_err(|_| "Failed to send request to channel")?;

            Ok(unsafe { std::mem::zeroed() })
        } else {
            let (response_sender, response_receiver) = oneshot::channel::<Response>();

            let channel_request = ChannelRequest {
                sender: Some(response_sender),
                data: Some(request),
            };

            self.sender
                .send(channel_request)
                .map_err(|_| "Failed to send request to channel")?;

            let response = response_receiver
                .await
                .map_err(|_| "Failed to receive response from channel")?;

            if TypeId::of::<()>() == response.type_id() && TypeId::of::<DST>() != TypeId::of::<()>()
            {
                return Err("Function expect some data but operation return no data".into());
            }

            response
                .downcast::<DST>()
                .map(|boxed| *boxed)
                .map_err(|_| "Failed to downcast response to expected type".into())
        }
    }

    pub fn make_weak(&self) -> ChannelWeakHandle {
        ChannelWeakHandle {
            region: self.region,
            id: self.id,
            sender: self.sender.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChannelWeakHandle {
    pub region: u32,
    pub id: u32,

    pub sender: tokio::sync::mpsc::UnboundedSender<ChannelRequest>,
}

impl ChannelWeakHandle {
    pub fn region(&self) -> u32 {
        self.region
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub async fn send<DST: std::any::Any + Send>(
        &self,
        request: ChannelCommand,
    ) -> Result<DST, Box<dyn std::error::Error + Send + Sync>> {
        if TypeId::of::<()>() == TypeId::of::<DST>() {
            let channel_request = ChannelRequest {
                sender: None,
                data: Some(request),
            };

            self.sender
                .send(channel_request)
                .map_err(|_| "Failed to send request to channel")?;

            Ok(unsafe { std::mem::zeroed() })
        } else {
            let (response_sender, response_receiver) = oneshot::channel::<Response>();

            let channel_request = ChannelRequest {
                sender: Some(response_sender),
                data: Some(request),
            };

            self.sender
                .send(channel_request)
                .map_err(|_| "Failed to send request to channel")?;

            let response = response_receiver
                .await
                .map_err(|_| "Failed to receive response from channel")?;

            if TypeId::of::<()>() == response.type_id() && TypeId::of::<DST>() != TypeId::of::<()>()
            {
                return Err("Function expect some data but operation return no data".into());
            }

            response
                .downcast::<DST>()
                .map(|boxed| *boxed)
                .map_err(|_| "Failed to downcast response to expected type".into())
        }
    }
}

pub async fn process_command(
    channel: &mut channel::Channel,
    mut request: ChannelRequest,
    _token: &tokio_util::sync::CancellationToken,
    _sender: &UnboundedSender<ChannelRequest>,
    weak: &mut Option<ChannelWeakHandle>,
) {
    let Some(data) = request.data.take() else {
        log::info!("Received ChannelRequest with no data");
        return;
    };

    match data {
        ChannelCommand::Heartbeat => {
            channel.heartbeat().await;
        }

        ChannelCommand::SetWeakHandle { handle } => {
            *weak = Some(handle);
        }

        ChannelCommand::BroadcastEvent {
            id,
            event,
            exception,
        } => {
            channel.users.broadcast_event(id, event, exception);
        }

        ChannelCommand::Connect { user } => {
            request.send(channel.connect(&user).await);
        }
        ChannelCommand::Disconnect { user_id } => {
            request.send(channel.disconnect(user_id).await);
        }
        ChannelCommand::RequestServerList => {
            request.send(channel.get_server_list());
        }
        ChannelCommand::SetClientList {
            user_id,
            client_ids,
        } => {
            let mut counter = 0;

            for id in &client_ids {
                let result = channel
                    .lists
                    .iter()
                    .find(|ojn| ojn.songid == id.songid() as i32);

                if !result.is_none() {
                    counter += 1;
                }
            }

            let (user, _) = channel
                .get_user_mut(user_id)
                .expect("User not found for SetClientList");

            log::info!(
                "User {} set client list with {} valid entries out of {}",
                user.nickname(),
                counter,
                client_ids.len()
            );

            user.set_music_list(client_ids);
        }
        ChannelCommand::GetRooms => {
            request.send(channel.get_rooms().await);
        }
        ChannelCommand::GetUsers => {
            request.send(channel.get_users());
        }
        ChannelCommand::SyncUserInfo { user_id } => {
            let Some((user, _)) = channel.get_user_mut(user_id) else {
                log::info!("User {} not found for sync", user_id);
                return;
            };

            user.sync().await;

            request.send(user.clone());
        }
        ChannelCommand::CreateRoom {
            user_id,
            name,
            password,
            mode,
            min_level,
            max_level,
        } => {
            let Some(channel_handle) = weak.as_ref() else {
                log::info!("Weak handle not set for channel");
                return;
            };

            let handle = channel
                .create_room(
                    channel_handle,
                    user_id,
                    name,
                    password,
                    mode,
                    min_level,
                    max_level,
                )
                .await;

            request.send(handle);
        }
        ChannelCommand::JoinRoom {
            room_id,
            user_id,
            password,
        } => {
            let response = channel.join_room(room_id, user_id, password).await;

            request.send(response);
        }
        ChannelCommand::LeaveRoom { user_id } => {
            let response = channel.leave_room(user_id).await;

            request.send(response);
        }
        ChannelCommand::Kicked { user_id } => {
            let Some(user) = channel.users.get_mut(user_id) else {
                log::info!("User {} not found for Kicked", user_id);
                return;
            };

            user.room_id = channel::INVALID_ROOM_ID;
        }
        ChannelCommand::Chat { user_id, message } => {
            channel.chat(user_id, message);
        }
        ChannelCommand::RequestOJNInfo { id } => {
            let result = channel
                .lists
                .iter()
                .find(|ojn| ojn.songid == id.songid() as i32)
                .map(|ojn| ojn.clone());

            request.send(result);
        }
        ChannelCommand::EquipItem {
            user_id,
            character_slot,
            item_slot,
        } => {
            let Some((user, _)) = channel.get_user_mut(user_id) else {
                log::info!("User {} not found for EquipItem", user_id);
                return;
            };

            let itemlist = crate::itemlist::get();

            let item = match user.get_item_from_slot(item_slot) {
                Some(item) => item,
                None => {
                    return request.send(InventoryEquipResponse {
                        result: 1,
                        ..Default::default()
                    });
                }
            };

            if let Some(info) = itemlist.get_item(item.id) {
                let chara_item_gender = match user.info.gender {
                    database::CharacterGender::Female => ItemGender::Female,
                    database::CharacterGender::Male => ItemGender::Male,
                };

                if info.gender != chara_item_gender && info.gender != ItemGender::Any {
                    return request.send(InventoryEquipResponse {
                        result: 1,
                        ..Default::default()
                    });
                }
            } else {
                return request.send(InventoryEquipResponse {
                    result: 1,
                    ..Default::default()
                });
            }

            let old = match user.set_equipment(character_slot, item.id) {
                Some(old) => old,
                None => {
                    return request.send(InventoryEquipResponse {
                        result: 1,
                        ..Default::default()
                    });
                }
            };

            // Equipment always has amount of one.
            user.set_item_in_slot(item_slot, ItemId::new(old));

            let _ = user.save().await;

            request.send((
                InventoryEquipResponse {
                    result: 0,
                    character_slot,
                    new_equip_item_id: item.id,
                    inventory_item_id: item_slot,
                    old_equip_item_id: old,
                },
                user.clone(),
            ));
        }
    }
}

pub async fn make_channel(
    cancelation_token: tokio_util::sync::CancellationToken,
    region: u32,
    id: u32,
    max_users: usize,
    path: String,
) -> Result<(ChannelHandle, JoinHandle<()>), Box<dyn std::error::Error + Send + Sync>> {
    let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<ChannelRequest>();
    let (mut channel, counter) = channel::Channel::new(region, id, &path).await;

    let sender_for_room = sender.clone();

    let (tx, rx) = tokio::sync::oneshot::channel::<()>();

    let join_handle = crate::util::spawn_named(
        &format!("Channel Region: {}, Id: {}", region, id),
        async move {
            log::info!(
                "Channel {}:{} started with {} lists",
                region,
                id,
                channel.lists.len()
            );

            tx.send(()).expect("Failed to send channel ready signal");

            let mut weak_handle = None;

            loop {
                tokio::select! {
                    Some(request) = receiver.recv() => {
                        let unwind_safe = std::panic::AssertUnwindSafe(process_command(
                            &mut channel,
                            request,
                            &cancelation_token,
                            &sender_for_room,
                            &mut weak_handle
                        ));

                        if unwind_safe.catch_unwind().await.is_err() {
                            log::info!("Channel {}:{} panicked while processing a request", region, id);
                        }
                    }
                    _ = cancelation_token.cancelled() => {
                        channel.shutdown().await;
                        break;
                    }
                }
            }

            log::info!("Channel {}:{} has been shut down", region, id);
        },
    );

    rx.await.expect("Failed to receive channel ready signal");

    let handle = ChannelHandle {
        region,
        id,
        sender,
        max_users,
        counter,
    };

    let _ = handle
        .send::<()>(ChannelCommand::SetWeakHandle {
            handle: handle.make_weak(),
        })
        .await
        .map_err(|e| format!("Failed to set weak handle for channel: {}", e))?;

    Ok((handle, join_handle))
}

pub enum ChannelCommand {
    Heartbeat,

    SetWeakHandle {
        handle: ChannelWeakHandle,
    },

    BroadcastEvent {
        id: EventId,
        event: Arc<dyn IEventData>,
        exception: Option<u64>,
    },
    RequestOJNInfo {
        id: MusicId,
    },

    Connect {
        user: User,
    },
    Disconnect {
        user_id: u64,
    },

    // ListRoom
    SyncUserInfo {
        user_id: u64,
    },
    RequestServerList,
    SetClientList {
        user_id: u64,
        client_ids: Vec<MusicId>,
    },
    GetRooms,
    GetUsers,
    CreateRoom {
        user_id: u64,
        name: String,
        password: Option<String>,
        mode: RoomMode,
        min_level: u8,
        max_level: u8,
    },
    JoinRoom {
        room_id: u32,
        user_id: u64,
        password: Option<String>,
    },
    LeaveRoom {
        user_id: u64,
    },
    Kicked {
        user_id: u64,
    },

    // Chat
    Chat {
        user_id: u64,
        message: String,
    },

    // MyRoom
    EquipItem {
        user_id: u64,
        character_slot: u32,
        item_slot: u32,
    },
}
