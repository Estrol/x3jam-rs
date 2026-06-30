pub mod room;
pub mod arena;
pub mod difficulty;
pub mod eventtype;
pub mod mode;
pub mod modifier;
pub mod music;
pub mod skill;
pub mod speed;
pub mod status;
pub mod team;
pub mod roomlist;

use std::{any::TypeId, sync::{Arc, atomic::AtomicUsize}};

pub use arena::RoomArena;
pub use difficulty::RoomDifficulty;
pub use eventtype::GameEventType;
use futures::FutureExt as _;
pub use mode::RoomMode;
pub use modifier::{ModifierReport, Modifiers};
pub use music::{MusicId, MusicIdEntry};
pub use skill::SkillId;
pub use speed::RoomSpeed;
pub use status::RoomStatus;
pub use team::TeamId;
pub use room::Room;
use tokio::sync::oneshot;

use crate::{channel::ChannelWeakHandle, gateway::routes::{game::{GameEventPingRequest, ScoreSubmitRequest}, listroom::RoomEntry}, user::User};

pub enum RoomCommand {
    GetEntryInfo,

    JoinRoom {
        user: User,
        password: Option<String>,
    },
    LeaveRoom {
        user_id: u64,
    },

    RoomChat {
        user_id: u64,
        message: String,
    },

    SetMusicId {
        music_id: MusicId,
        difficulty: RoomDifficulty,
        speed: RoomSpeed,
    },
    SetName {
        name: String,
    },
    SetArena {
        arena: RoomArena,
    },
    SetReady {
        user_id: u64,
    },
    SetSkills {
        skills: Vec<SkillId>,
    },
    SetTeam {
        user_id: u64,
        team: TeamId,
    },
    SetModifier {
        modifier: Modifiers,
        value: u32,
    },
    SetAllModifiers {
        modifiers: ModifierReport,
    },

    StartGame {
        user_id: u64,
    },
    LeaveGame {
        user_id: u64,
    },
    GameEvent {
        user_id: u64,
        event: GameEventPingRequest,
    },
    ConfirmGameLoaded {
        user_id: u64,
    },
    SubmitScore {
        user_id: u64,
        score_request: ScoreSubmitRequest,
    },
    Chat {
        user_id: u64,
        message: String,
    },
}

pub type Response = Box<dyn std::any::Any + Send + Sync>;

pub struct RoomRequest {
    pub sender: Option<oneshot::Sender<Response>>,
    pub data: Option<RoomCommand>,
}

impl RoomRequest {
    pub fn send<DST: std::any::Any + Send + Sync + 'static>(mut self, response: DST) {
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(Box::new(response));
        }
    }
}

impl Drop for RoomRequest {
    fn drop(&mut self) {
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(Box::new(()));
        }
    }
}

#[derive(Debug)]
pub struct RoomHandle {
    pub id: u32,

    pub counter: Arc<AtomicUsize>,
    pub sender: tokio::sync::mpsc::UnboundedSender<RoomRequest>,
    pub join_handle: tokio::task::JoinHandle<()>,
}

impl RoomHandle {
    pub async fn send<DST: std::any::Any + Send>(
        &self,
        request: RoomCommand,
    ) -> Result<DST, Box<dyn std::error::Error + Send + Sync>> {
        if TypeId::of::<()>() == TypeId::of::<DST>() {
            let channel_request = RoomRequest {
                sender: None,
                data: Some(request),
            };

            self.sender
                .send(channel_request)
                .map_err(|_| "Failed to send request to channel")?;

            // SAFETY: We are returning a zeroed value for the type DST, which is expected to be () in this case. 
            // This is safe because we are not actually using the value, and it will be ignored by the caller.
            Ok(unsafe { std::mem::zeroed() })
        } else {
            let (response_sender, response_receiver) = oneshot::channel::<Response>();

            let channel_request = RoomRequest {
                sender: Some(response_sender),
                data: Some(request),
            };

            self.sender
                .send(channel_request)
                .map_err(|_| "Failed to send request to channel")?;

            let response = response_receiver
                .await
                .map_err(|_| "Failed to receive response from channel")?;

            if TypeId::of::<()>() == response.type_id() && TypeId::of::<DST>() != TypeId::of::<()>() {
                return Err("Function expect some data but operation return no data".into());
            }

            response
                .downcast::<DST>()
                .map(|boxed| *boxed)
                .map_err(|_| "Failed to downcast response to expected type".into())
        }
    }

    pub async fn send_timed<DST: std::any::Any + Send>(
        &self,
        request: RoomCommand,
        timeout: std::time::Duration,
    ) -> Result<DST, Box<dyn std::error::Error + Send + Sync>> {
        let send_future = self.send::<DST>(request);
        let timeout_future = tokio::time::sleep(timeout);

        tokio::select! {
            result = send_future => result,
            _ = timeout_future => Err("Request timed out".into()),
        }
    }

    pub fn make_weak(&self) -> RoomWeakHandle {
        RoomWeakHandle {
            id: self.id,
            sender: self.sender.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RoomWeakHandle {
    pub id: u32,
    pub sender: tokio::sync::mpsc::UnboundedSender<RoomRequest>,
}

impl RoomWeakHandle {
    pub async fn send<DST: std::any::Any + Send>(
        &self,
        request: RoomCommand,
    ) -> Result<DST, Box<dyn std::error::Error + Send + Sync>> {
        if TypeId::of::<()>() == TypeId::of::<DST>() {
            let channel_request = RoomRequest {
                sender: None,
                data: Some(request),
            };

            self.sender
                .send(channel_request)
                .map_err(|_| "Failed to send request to channel")?;

            // SAFETY: We are returning a zeroed value for the type DST, which is expected to be () in this case. 
            // This is safe because we are not actually using the value, and it will be ignored by the caller.
            Ok(unsafe { std::mem::zeroed() })
        } else {
            let (response_sender, response_receiver) = oneshot::channel::<Response>();

            let channel_request = RoomRequest {
                sender: Some(response_sender),
                data: Some(request),
            };

            self.sender
                .send(channel_request)
                .map_err(|_| "Failed to send request to channel")?;

            let response = response_receiver
                .await
                .map_err(|_| "Failed to receive response from channel")?;

            if TypeId::of::<()>() == response.type_id() && TypeId::of::<DST>() != TypeId::of::<()>() {
                return Err("Function expect some data but operation return no data".into());
            }

            response
                .downcast::<DST>()
                .map(|boxed| *boxed)
                .map_err(|_| "Failed to downcast response to expected type".into())
        }
    }
}

pub async fn make_room(
    caller: &User,
    channel_handle: ChannelWeakHandle,
    cancellation_token: tokio_util::sync::CancellationToken,

    id: u32,
    name: String,
    password: Option<String>,
    mode: RoomMode,
    min_lvl: u8,
    max_lvl: u8,
) -> Result<RoomHandle, Box<dyn std::error::Error + Send + Sync>> {
    let counter = Arc::new(AtomicUsize::new(0));
    let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<RoomRequest>();

    let mut room = Room::new(caller, counter.clone(), channel_handle, id, name, password, mode, min_lvl, max_lvl);

    let handle = tokio::spawn(async move {
        loop {
            tokio::select! {
                Some(request) = receiver.recv() => {
                    let unwind_safe = std::panic::AssertUnwindSafe(process_command(&mut room, request));
                    if unwind_safe.catch_unwind().await.is_err() {
                        println!("Room {} panicked while processing command", id);
                    }
                }
                _ = cancellation_token.cancelled() => {
                    println!("Room {} is being cancelled", id);
                    break;
                }
            }
        }
    });

    Ok(RoomHandle { id, counter, sender, join_handle: handle })
}

pub struct RoomListRepository {
    pub rooms: Vec<Room>,
}

pub async fn process_command(room: &mut Room, mut command: RoomRequest) {
    let Some(data) = command.data.take() else {
        println!("Room command data is None");
        return;
    };

    match data {
        RoomCommand::GetEntryInfo => {
            command.send(RoomEntry {
                id: room.id,
                state: room.status,
                name: to_cstring(&room.title),
                is_password: room.password.is_some(),
                ojn_id: room.music_id,
                difficulty: room.difficulty,
                mode: room.mode,
                speed: room.speed,
                max_players: room.max_players() as u8,
                current_players: room.player_count() as u8,
                min_level: room.min_level,
                max_level: room.max_level,
                skills: room.skill_slot.clone(),
                premium: 0
            });
        }
        RoomCommand::RoomChat { user_id, message } => {
            room.chat(user_id, &message).await;
        }
        RoomCommand::SetMusicId {
            music_id,
            difficulty,
            speed,
        } => {
            room
                .set_music_id(music_id, difficulty, speed)
                .await;
        }
        RoomCommand::SetName { name } => {
            room.set_name(&name).await;
        }
        RoomCommand::SetArena { arena } => {
            room.set_arena(arena);
        }
        RoomCommand::SetReady { user_id } => {
            room.set_ready(user_id).await;
        }
        RoomCommand::SetSkills { skills } => {
            room.set_skills(skills).await;
        }
        RoomCommand::SetTeam { user_id, team } => {
            room.set_team(user_id, team).await;
        }
        RoomCommand::SetModifier {
            modifier,
            value,
        } => {
            room.set_modifier(modifier, value).await;
        }
        RoomCommand::SetAllModifiers { modifiers } => {
            room.set_all_modifiers(modifiers).await;
        }
        RoomCommand::StartGame { user_id } => {
            command.send(room.start_game(user_id).await);
        }
        RoomCommand::JoinRoom { user, password } => {
            command.send(room.add_user(&user, password).await);
        },
        RoomCommand::LeaveRoom { user_id } => {
            command.send(room.remove_user(user_id).await);
        },
        RoomCommand::LeaveGame { user_id } => {
            command.send(room.leave_game(user_id).await);
        },
        RoomCommand::GameEvent { user_id, event } => {
            room.on_game_event(user_id, event).await;
        },
        RoomCommand::ConfirmGameLoaded { user_id } => {
            room.confirm_game_loaded(user_id);
        },
        RoomCommand::SubmitScore { user_id, score_request } => {
            command.send(room.on_game_score_submit(user_id, score_request).await);
        },
        RoomCommand::Chat { user_id, message } => {
            room.chat(user_id, &message).await;
        },
    }
}

pub fn to_cstring(t: &str) -> std::ffi::CString {
    std::ffi::CString::new(t).expect("Failed to convert to CString")
}