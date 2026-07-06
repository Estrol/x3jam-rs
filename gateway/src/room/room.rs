use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::cmp::Reverse;

#[cfg(not(feature = "disable-o2hook2-mod"))]
use std::collections::HashMap;

pub use super::arena::RoomArena;
pub use super::difficulty::RoomDifficulty;
pub use super::eventtype::GameEventType;
pub use super::mode::RoomMode;
pub use super::modifier::{ModifierReport, Modifiers};
pub use super::music::MusicId;
pub use super::skill::SkillId;
pub use super::speed::RoomSpeed;
pub use super::status::RoomStatus;
pub use super::team::TeamId;

use crate::channel::ojnlist::Header;
use crate::channel::{ChannelCommand, ChannelWeakHandle};
use crate::gateway::events::listroom::ListRoomChangeRoomMaxPlayerEventArgs;
use crate::gateway::events::room::{
    PlayingState, RoomOnMusicStateChangedEventArgs, RoomOnSlotChangedEventArgs,
};
use crate::gateway::routes::game::ScoreSubmitResponse;
use crate::gateway::routes::listroom::{
    JoinErrorCode, JoinRoomResponse, MemberInfo, PositionInfo, PositionStatus,
};
use crate::{
    gateway::{
        commands::EventId,
        events::{
            IEventData,
            game::{
                GameFinishEventArgs, GameOnLoadingReadyEventArgs, GameOnNoteEventEventArgs,
                GameOnPlayerLeaveEventArgs, PlayerResult, PlayerScore, SCORE_INVALID, SCORE_VALID,
            },
            listroom::{
                ListRoomChangeRoomMusicIdEventArgs, ListRoomChangeRoomNameEventArgs,
                ListRoomChangeRoomSkillEventArgs, ListRoomChangeRoomStatusEventArgs,
            },
            room::{
                RoomOnArenaChangedEventArgs,
                RoomOnChatEventArgs, RoomOnGameStartEventArgs,
                RoomOnMusicIdChangedEventArgs, RoomOnNameChangedEventArgs,
                RoomOnPlayerEnterEventArgs, RoomOnPlayerLeaveEventArgs, RoomOnReadyEventArgs,
                RoomOnSkillChangedEventArgs, RoomOnTeamChangedEventArgs,
            },
        },
        routes::{
            game::{GameEventPingRequest, ScoreSubmitRequest},
            room::GameStartResult,
        },
    },
    user::User,
};

#[cfg(not(feature = "disable-o2hook2-mod"))]
use crate::gateway::events::room::{RoomOnAllModifiersChangedEventArgs, RoomOnModifierChangedEventArgs};

#[derive(Debug, Clone, Default)]
pub struct GameData {
    pub finished: bool,
    pub loaded: bool,

    pub health: u16,
    pub cool: i16,
    pub good: i16,
    pub bad: i16,
    pub miss: i16,
    pub max_combo: i16,
    pub jam_combo: i16,
    pub score: i32,
}

#[derive(Debug, Clone)]
pub enum UserSlot {
    Available,
    User {
        id: u64,
        join_pos: u64,

        team: TeamId,
        host: bool,
        ready: bool,
        data: GameData,
        state: PlayingState,

        user: User,
    },
    Locked,
}

macro_rules! all_active_players {
    ($players:expr, $pattern:pat => $cond:expr) => {
        $players.iter().all(|p| match p {
            $pattern => $cond,
            _ => true,
        })
    };
}

pub enum ModifierValue {
    Float(f32),
    U32(u32),
}

pub struct Room {
    pub id: u32,
    pub counter: Arc<AtomicUsize>,
    pub channel_handle: ChannelWeakHandle,

    pub title: String,
    pub password: Option<String>,
    pub color: TeamId,

    pub mode: RoomMode,
    pub difficulty: RoomDifficulty,
    pub arena: RoomArena,
    pub arena_random: bool,
    pub speed: RoomSpeed,
    pub music_id: MusicId,
    pub header: Option<Header>,
    pub status: RoomStatus,

    pub skill_slot: Vec<SkillId>,
    pub skill_seed: u32,
    pub players: [UserSlot; 8],

    pub min_level: u8,
    pub max_level: u8,

    #[cfg(not(feature = "disable-o2hook2-mod"))]
    pub modifiers: HashMap<Modifiers, ModifierValue>,

    pub join_pos_counter: u64, // Counter to assign join positions to players
}

impl Room {
    pub fn new(
        user: &User,
        counter: Arc<AtomicUsize>,
        channel_handle: ChannelWeakHandle,

        id: u32,
        title: String,
        password: Option<String>,
        mode: RoomMode,
        min_level: u8,
        max_level: u8,
    ) -> (Self, ModifierReport) {
        let mut room = Room {
            id,
            counter,
            channel_handle,
            title,
            password,
            color: TeamId::Red,
            mode,
            difficulty: RoomDifficulty::Hard,
            arena: RoomArena::ARENA1,
            arena_random: false,
            speed: RoomSpeed::Speed10,
            music_id: MusicId::new(0),
            header: None,
            status: RoomStatus::Waiting,
            skill_slot: Vec::new(),
            skill_seed: gen_random_seed(),
            max_level,
            min_level,
            #[cfg(not(feature = "disable-o2hook2-mod"))]
            modifiers: HashMap::from_iter([
                (Modifiers::Rate, ModifierValue::Float(1.0)),
                (Modifiers::Fln, ModifierValue::U32(0)),
                (Modifiers::Sln, ModifierValue::U32(0)),
                (Modifiers::Nln, ModifierValue::U32(0)),
                (Modifiers::Timing, ModifierValue::U32(0)),
            ]),
            players: [const { UserSlot::Available }; 8],
            join_pos_counter: 1,
        };

        room.players[0] = UserSlot::User {
            id: user.id,
            host: true,
            ready: true,
            team: room.next_color(),
            data: GameData::default(),
            user: user.clone(),
            state: PlayingState::Waiting,
            join_pos: 0,
        };

        room.counter.store(1, std::sync::atomic::Ordering::SeqCst);

        #[cfg(not(feature = "disable-o2hook2-mod"))]
        {
            let modifier = room.get_modifier_report();
            (room, modifier)
        }
        #[cfg(feature = "disable-o2hook2-mod")]
        {
            (room, ModifierReport::default())
        }
    }

    pub async fn heartbeat(&mut self) {
        let disconnected: Vec<u64> = self
            .players
            .iter()
            .filter_map(|slot| {
                if let UserSlot::User { user, .. } = slot {
                    if let Some(sender) = user.sender.as_ref() {
                        if sender.is_closed() {
                            return Some(user.id);
                        } else {
                            return None;
                        }
                    } else {
                        return Some(user.id);
                    }
                }

                None
            })
            .collect();

        for user_id in disconnected {
            self.remove_user(user_id).await;
        }
    }

    pub fn get_user_slot_index(&self, user: u64) -> Option<usize> {
        self.players.iter().position(
            |p| matches!(p, UserSlot::User { user: player_user, .. } if player_user.id == user),
        )
    }

    pub fn get_user_slot(&self, user: u64) -> Option<(usize, &UserSlot)> {
        self.players.iter().enumerate().find_map(|(i, p)| {
            if let UserSlot::User {
                user: player_user, ..
            } = p
            {
                if player_user.id == user {
                    return Some((i, p));
                }
            }
            None
        })
    }

    pub fn get_user_slot_mut(&mut self, user: u64) -> Option<(usize, &mut UserSlot)> {
        self.players.iter_mut().enumerate().find_map(|(i, p)| {
            if let UserSlot::User {
                user: player_user, ..
            } = p
            {
                if player_user.id == user {
                    return Some((i, p));
                }
            }
            None
        })
    }

    pub fn next_color(&mut self) -> TeamId {
        self.color = match self.color {
            TeamId::Red => TeamId::Orange,
            TeamId::Orange => TeamId::Yellow,
            TeamId::Yellow => TeamId::Green,
            TeamId::Green => TeamId::Cyan,
            TeamId::Cyan => TeamId::Blue,
            TeamId::Blue => TeamId::Purple,
            TeamId::Purple => TeamId::DarkRed,
            TeamId::DarkRed => TeamId::Red,
        };
        self.color
    }

    pub fn get_next_counter(&mut self) -> u64 {
        let current = self.join_pos_counter;
        self.join_pos_counter += 1;
        current
    }

    pub fn find_same_user(&self, user: &User) -> Option<usize> {
        self.players.iter().position(|p| match p {
            UserSlot::User {
                user: player_user, ..
            } => player_user.id == user.id,
            _ => false,
        })
    }

    pub async fn add_user(
        &mut self,
        user: &User,
        password: Option<String>,
    ) -> (JoinRoomResponse, Option<ModifierReport>) {
        if self.password.is_some() && self.password != password {
            return (JoinRoomResponse::invalid_password(), None);
        }

        let (slot_idex, team) = {
            let slot = self
                .players
                .iter()
                .position(|p| matches!(p, UserSlot::Available));

            let Some(slot) = slot else {
                println!("No available slots in room {}", self.id);
                return (JoinRoomResponse::room_full(), None);
            };

            if let Some(_) = self.find_same_user(user) {
                return (JoinRoomResponse::user_not_found(), None);
            }

            let color = self.next_color();
            let join_pos = self.get_next_counter();

            self.players[slot] = UserSlot::User {
                id: user.id,
                host: false,
                ready: false,
                team: color,
                data: GameData::default(),
                user: user.clone(),
                state: PlayingState::Waiting,
                join_pos,
            };

            self.broadcast(
                EventId::RoomOnPlayerEnter,
                RoomOnPlayerEnterEventArgs {
                    slot: slot as u8,
                    nickname: to_cstring(&user.nickname()),
                    level: user.level(),
                    gender: 1,
                    team: color,
                    unk1: 0,
                    equipment: user.equipment(),
                    music_list: user.music_list(),
                },
                Some(&user),
            );

            self.counter = Arc::new(AtomicUsize::new(self.counter.load(Ordering::SeqCst) + 1));

            (slot, color)
        };

        let mut slots = [const {
            PositionInfo {
                position: 0,
                status: PositionStatus::Empty,
                member: None,
            }
        }; 7];

        let mut index = 0;

        #[cfg(not(feature = "disable-o2hook2-mod"))]
        let modifier = self.get_modifier_report();
        #[cfg(feature = "disable-o2hook2-mod")]
        let modifier = ModifierReport::default();

        for (i, user) in self.players.iter().enumerate() {
            if i == slot_idex as usize {
                continue;
            }

            let slot = &mut slots[index];
            slot.position = i as u8;

            match user {
                UserSlot::Available => slot.status = PositionStatus::Empty,
                UserSlot::User {
                    team,
                    host,
                    ready,
                    user,
                    state,
                    ..
                } => {
                    slot.status = PositionStatus::Occupied;

                    let nickname = std::ffi::CString::new(user.nickname())
                        .unwrap_or_else(|_| std::ffi::CString::new("InvalidNickname").unwrap());

                    let member_info = MemberInfo {
                        nickname,
                        level: user.level(),
                        gender: 1,
                        is_room_master: *host,
                        color: *team,
                        ready: *ready,
                        state: *state,
                        equipment: user.equipment(),
                        list: user.music_list(),
                    };

                    slot.member = Some(member_info);
                }
                UserSlot::Locked => slot.status = PositionStatus::Locked,
            }

            index += 1;
        }

        let response = JoinRoomResponse {
            result: JoinErrorCode::Success,
            slot: slot_idex as u8,
            team: team,
            name: std::ffi::CString::new(self.title.clone()).unwrap_or_default(),
            music_id: self.music_id,
            arena: self.arena,
            mode: self.mode,
            diffculty: self.difficulty,
            speed: self.speed,
            user_count: 7,
            slots,
            skills: self.skill_slot.clone(),
            premium: 0,
        };

        self.broadcast_channel(
            EventId::ListRoomOnRoomPlayerCountChanged,
            ListRoomChangeRoomMaxPlayerEventArgs {
                id: self.id,
                max_player: self.max_players() as u8,
                current_player: self.player_count() as u8,
                premium: 0,
            },
            None,
        )
        .await;

        (response, Some(modifier))
    }

    pub async fn remove_user(&mut self, user: u64) -> Option<usize> {
        if let Some(slot) = self.get_user_slot_index(user) {
            if self.player_count() > 1 && self.status == RoomStatus::Playing {
                self.on_game_score_submit(user, ScoreSubmitRequest::default())
                    .await;
            }

            let player_count = self.player_count();

            let host = matches!(self.players[slot], UserSlot::User { host: true, .. });
            self.players[slot] = UserSlot::Available;

            // If previously there more than 1.
            if player_count > 1 {
                if host {
                    let next_host_idx = self
                        .players
                        .iter()
                        .enumerate()
                        .filter_map(|(i, p)| {
                            if let UserSlot::User { join_pos, .. } = p {
                                Some((i, *join_pos))
                            } else {
                                None
                            }
                        })
                        .min_by_key(|&(_, pos)| pos)
                        .map(|(i, _)| i)
                        .expect("Room is not empty, but failed to find a new host");

                    if let UserSlot::User { host, .. } = &mut self.players[next_host_idx] {
                        *host = true;
                    }
                }

                let host = self.get_host_slot();

                self.broadcast(
                    EventId::RoomOnPlayerLeave,
                    RoomOnPlayerLeaveEventArgs {
                        slot: slot as u8,
                        room_master_slot: host as u8,
                        premium: 0,
                    },
                    None,
                );

                println!(
                    "User {} left room {} (slot {}), new host is slot {}",
                    user, self.id, slot, host
                );
            } else {
                println!(
                    "User {} left room {} (slot {}), room is now empty",
                    user, self.id, slot
                );
            }

            Some(self.player_count())
        } else {
            println!("User is not in room {}", self.id);
            None
        }
    }

    pub fn is_host(&self, user: u64) -> bool {
        self.get_user_slot(user)
            .map(|(_, slot)| matches!(slot, UserSlot::User { host: true, .. }))
            .unwrap_or(false)
    }

    pub fn get_host_slot(&self) -> usize {
        self.players
            .iter()
            .position(|p| matches!(p, UserSlot::User { host: true, .. }))
            .expect("Room has no host")
    }

    pub fn is_all_ready(&self) -> bool {
        all_active_players!(self.players, UserSlot::User { ready, host, .. } => *ready || *host)
    }

    pub fn is_everyone_finished(&self) -> bool {
        all_active_players!(self.players, UserSlot::User { data, .. } => data.finished)
    }

    pub fn is_loaded(&self) -> bool {
        all_active_players!(self.players, UserSlot::User { data, .. } => data.loaded)
    }

    pub async fn set_status(&mut self, status: RoomStatus) {
        self.status = status;

        self.broadcast_channel(
            EventId::ListRoomOnRoomStatusChanged,
            ListRoomChangeRoomStatusEventArgs {
                id: self.id,
                status,
            },
            None,
        )
        .await;
    }

    pub async fn set_ready(&mut self, user: u64) {
        if let Some((slot, UserSlot::User { ready, .. })) = self.get_user_slot_mut(user) {
            let ready_value = !*ready;
            *ready = ready_value;

            self.broadcast(
                EventId::RoomOnReadyChanged,
                RoomOnReadyEventArgs {
                    slot: slot as u8,
                    ready: ready_value,
                },
                None,
            );
        }
    }

    pub fn max_players(&self) -> usize {
        self.players
            .iter()
            .filter(|p| !matches!(p, UserSlot::Locked))
            .count()
    }

    pub fn player_count(&self) -> usize {
        self.players
            .iter()
            .filter(|p| matches!(p, UserSlot::User { .. }))
            .count()
    }

    pub async fn start_game(&mut self, user_id: u64) -> GameStartResult {
        if !self.is_host(user_id) {
            return GameStartResult::NotHost;
        }

        if !self.is_all_ready() {
            return GameStartResult::NotAllReady;
        }

        self.skill_seed = gen_random_seed();

        // Elegantly reset game data using standard Default trait
        for player in &mut self.players {
            if let UserSlot::User { data, .. } = player {
                *data = GameData::default();
            }
        }

        #[cfg(not(feature = "disable-o2hook2-mod"))]
        self.broadcast(
            EventId::RoomOnAllModifiersChanged,
            RoomOnAllModifiersChangedEventArgs {
                modifiers: self.get_modifier_report(),
            },
            None,
        );

        self.set_status(RoomStatus::Playing).await;

        self.broadcast(
            EventId::RoomOnGameStart,
            RoomOnGameStartEventArgs {
                success: GameStartResult::Success,
                seed: self.skill_seed,
            },
            None,
        );

        self.broadcast_channel(
            EventId::ListRoomOnRoomStatusChanged,
            ListRoomChangeRoomStatusEventArgs {
                id: self.id,
                status: self.status,
            },
            None,
        )
        .await;

        GameStartResult::Success
    }

    pub async fn set_name(&mut self, user_id: u64, name: &str) {
        if !self.is_host(user_id) {
            println!("User {} is not the host of room {}", user_id, self.id);
            return;
        }

        self.title = name.to_string();

        self.broadcast(
            EventId::RoomOnNameChanged,
            RoomOnNameChangedEventArgs {
                name: to_cstring(name),
            },
            None,
        );

        self.broadcast_channel(
            EventId::ListRoomOnRoomNameChanged,
            ListRoomChangeRoomNameEventArgs {
                id: self.id,
                name: to_cstring(&self.get_name_rate()),
            },
            None,
        )
        .await;
    }

    pub fn get_name_rate(&self) -> String {
        #[cfg(not(feature = "disable-o2hook2-mod"))]
        let title = if let Some(ModifierValue::Float(rate)) = self.modifiers.get(&Modifiers::Rate) {
            if *rate != 1.0 {
                format!("[{:.2}x] {}", rate, self.title)
            } else {
                self.title.clone()
            }
        } else {
            self.title.clone()
        };

        #[cfg(feature = "disable-o2hook2-mod")]
        let title = self.title.clone();

        title
    }

    pub async fn set_music_id(
        &mut self,
        user_id: u64,
        music_id: MusicId,
        difficulty: RoomDifficulty,
        speed: RoomSpeed,
    ) {
        if !self.is_host(user_id) {
            println!("User {} is not the host of room {}", user_id, self.id); // Request forged?
            return;
        }

        if self.music_id != music_id {
            let Ok(header) = self
                .channel_handle
                .send::<Option<Header>>(ChannelCommand::RequestOJNInfo { id: music_id })
                .await
            else {
                println!("Failed to fetch OJN info for music ID {}", music_id);
                return;
            };

            let Some(header) = header else {
                println!("OJN info not found for music ID {}", music_id);
                return;
            };

            self.header = Some(header);
        }

        self.music_id = music_id;
        self.difficulty = difficulty;
        self.speed = speed;

        self.broadcast(
            EventId::RoomOnMusicIdChanged,
            RoomOnMusicIdChangedEventArgs {
                id: music_id,
                difficulty,
                speed,
            },
            None,
        );

        self.broadcast_channel(
            EventId::ListRoomOnRoomMusicIdChanged,
            ListRoomChangeRoomMusicIdEventArgs {
                id: self.id,
                music_id,
                difficulty,
                speed,
            },
            None,
        )
        .await;
    }

    pub fn set_arena(&mut self, user_id: u64, arena: RoomArena) {
        if !self.is_host(user_id) {
            println!("User {} is not the host of room {}", user_id, self.id);
            return;
        }

        self.arena = arena;

        self.broadcast(
            EventId::RoomOnArenaChanged,
            RoomOnArenaChangedEventArgs {
                arena,
                random: self.arena_random,
            },
            None,
        );
    }

    pub async fn set_skills(&mut self, user_id: u64, ring: Vec<SkillId>) {
        if !self.is_host(user_id) {
            println!("User {} is not the host of room {}", user_id, self.id);
            return;
        }

        self.skill_slot = ring;

        self.broadcast(
            EventId::RoomOnSkillChanged,
            RoomOnSkillChangedEventArgs {
                ring: self.skill_slot.clone(),
            },
            None,
        );

        self.broadcast_channel(
            EventId::ListRoomOnRoomSkillChanged,
            ListRoomChangeRoomSkillEventArgs {
                id: self.id,
                skill: self.skill_slot.clone(),
            },
            None,
        )
        .await;
    }

    pub async fn set_team(&mut self, user: u64, team: TeamId) {
        if let Some((
            slot,
            UserSlot::User {
                team: player_team, ..
            },
        )) = self.get_user_slot_mut(user)
        {
            *player_team = team;

            self.broadcast(
                EventId::RoomOnTeamChanged,
                RoomOnTeamChangedEventArgs {
                    slot: slot as u8,
                    team,
                },
                None,
            );
        }
    }

    pub async fn on_game_event(&mut self, user: u64, event: GameEventPingRequest) {
        if let Some((slot, UserSlot::User { data, .. })) = self.get_user_slot_mut(user) {
            match event.r#type {
                GameEventType::Life => data.health = event.value,
                GameEventType::Jam => data.jam_combo = event.value as i16,
            }

            data.score = event.score as i32;

            self.update_and_broadcast_ranks(slot, event);
        }
    }

    fn update_and_broadcast_ranks(&mut self, slot_idx: usize, event: GameEventPingRequest) {
        let mut ranks = [0xFFu8; 8];

        if self.player_count() > 1 {
            let mut empty = true;

            for player in self.players.iter() {
                if let UserSlot::User { data, .. } = player {
                    if data.score > 0 {
                        empty = false;
                        break;
                    }
                }
            }

            if !empty {
                // Rank, slot, score
                let mut sorted_players: Vec<(usize, usize, i32)> = self
                    .players
                    .iter()
                    .enumerate()
                    .filter_map(|(i, player)| match player {
                        UserSlot::User { data, .. } => Some((0, i, data.score)),
                        _ => None,
                    })
                    .collect();

                sorted_players.sort_unstable_by_key(|&(_, _, score)| Reverse(score));

                for (rank, &(_, slot, _)) in sorted_players.iter().enumerate() {
                    ranks[slot] = rank as u8;
                }
            }
        }

        self.broadcast(
            EventId::GameOnNoteEvent,
            GameOnNoteEventEventArgs {
                slot: slot_idx as u8,
                r#type: event.r#type,
                value: event.value,
                ranks,
            },
            None,
        );
    }

    pub async fn on_game_score_submit(
        &mut self,
        user: u64,
        result: ScoreSubmitRequest,
    ) -> ScoreSubmitResponse {
        let max_notes = self.fetch_max_notes().await;

        #[cfg(not(feature = "disable-o2hook2-mod"))]
        let (rate, fln, sln, nln) = (
            self.modifiers
                .get(&Modifiers::Rate)
                .and_then(|v| match v {
                    ModifierValue::Float(f) => Some(*f),
                    _ => None,
                })
                .unwrap_or(1.0),
            self.modifiers
                .get(&Modifiers::Fln)
                .and_then(|v| match v {
                    ModifierValue::U32(v) => Some(*v),
                    _ => None,
                })
                .unwrap_or(0),
            self.modifiers
                .get(&Modifiers::Sln)
                .and_then(|v| match v {
                    ModifierValue::U32(v) => Some(*v),
                    _ => None,
                })
                .unwrap_or(0),
            self.modifiers
                .get(&Modifiers::Nln)
                .and_then(|v| match v {
                    ModifierValue::U32(v) => Some(*v),
                    _ => None,
                })
                .unwrap_or(0),
        );

        // Early return: Invert the check to avoid deep nesting
        let slot = match self.get_user_slot_mut(user) {
            Some((slot, UserSlot::User { data, .. })) => {
                #[cfg(not(feature = "disable-o2hook2-mod"))]
                if result.song_rate != rate
                    && result.fln != fln
                    && result.sln != sln
                    && result.nln != nln
                {
                    println!(
                        "[Warn] User {} submitted score with mismatched modifiers: expected rate {}, fln {}, sln {}, nln {}, but got rate {}, fln {}, sln {}, nln {}",
                        user,
                        rate,
                        fln,
                        sln,
                        nln,
                        result.song_rate,
                        result.fln,
                        result.sln,
                        result.nln
                    );
                    return ScoreSubmitResponse {
                        slot: slot as u8,
                        success: false,
                    };
                }

                if data.finished {
                    return ScoreSubmitResponse {
                        slot: slot as u8,
                        success: false,
                    };
                }

                let user_total_notes = result.cool as i32
                    + result.good as i32
                    + result.bad as i32
                    + result.miss as i32;
                if user_total_notes > max_notes {
                    return ScoreSubmitResponse {
                        slot: slot as u8,
                        success: false,
                    };
                }

                // Update user state
                data.finished = true;
                data.cool = result.cool;
                data.good = result.good;
                data.bad = result.bad;
                data.miss = result.miss;
                data.max_combo = result.max_combo;
                data.jam_combo = result.jam_combo;
                data.score = result.score;

                slot
            }
            _ => {
                println!("[Warn] User {} is not in room {}", user, self.id);
                return ScoreSubmitResponse {
                    slot: u8::MAX, // Invalid slot to indicate error
                    success: false,
                };
            }
        };

        // Orchestrate match finish if applicable
        if self.is_everyone_finished() {
            self.process_match_results(max_notes).await;
        }

        ScoreSubmitResponse {
            slot: slot as u8,
            success: true,
        }
    }

    async fn fetch_max_notes(&self) -> i32 {
        self.header
            .as_ref()
            .map(|h| match self.difficulty {
                RoomDifficulty::Easy => h.note_count[0],
                RoomDifficulty::Normal => h.note_count[1],
                RoomDifficulty::Hard => h.note_count[2],
            })
            .unwrap_or(0)
    }

    async fn process_match_results(&mut self, max_notes: i32) {
        // Rank players efficiently (sort descending)
        let mut positions: Vec<(usize, i32)> = self
            .players
            .iter()
            .enumerate()
            .filter_map(|(i, p)| match p {
                UserSlot::User { data, .. } if data.finished => Some((i, data.score)),
                _ => None,
            })
            .collect();

        positions.sort_unstable_by_key(|&(_, score)| std::cmp::Reverse(score));

        let mut ranks = vec![0; self.players.len()];
        for (rank, &(slot, _)) in positions.iter().enumerate() {
            ranks[slot] = rank as u32;
        }

        let mut results = Vec::with_capacity(self.players.len());
        let mut scores_to_save = Vec::new();
        let timestamp = chrono::Utc::now().to_utc();
        let skill_ids: Vec<u32> = self.skill_slot.iter().map(|s| s.0).collect(); // Compute once

        // let rate = match self.modifiers.get(&Modifiers::Rate) {
        //     Some(ModifierValue::Float(f)) => *f,
        //     _ => 1.0,
        // };

        // let timing = match self.modifiers.get(&Modifiers::Timing) {
        //     Some(ModifierValue::U32(v)) => *v,
        //     _ => 0,
        // };

        // let fln = match self.modifiers.get(&Modifiers::Fln) {
        //     Some(ModifierValue::U32(v)) => *v,
        //     _ => 0,
        // };

        // let sln = match self.modifiers.get(&Modifiers::Sln) {
        //     Some(ModifierValue::U32(v)) => *v,
        //     _ => 0,
        // };

        // let nln = match self.modifiers.get(&Modifiers::Nln) {
        //     Some(ModifierValue::U32(v)) => *v,
        //     _ => 0,
        // };

        #[cfg(not(feature = "disable-o2hook2-mod"))]
        let (rate, fln, sln, nln, timing) = (
            self.modifiers
                .get(&Modifiers::Rate)
                .and_then(|v| match v {
                    ModifierValue::Float(f) => Some(*f),
                    _ => None,
                })
                .unwrap_or(1.0),
            self.modifiers
                .get(&Modifiers::Fln)
                .and_then(|v| match v {
                    ModifierValue::U32(v) => Some(*v),
                    _ => None,
                })
                .unwrap_or(0),
            self.modifiers
                .get(&Modifiers::Sln)
                .and_then(|v| match v {
                    ModifierValue::U32(v) => Some(*v),
                    _ => None,
                })
                .unwrap_or(0),
            self.modifiers
                .get(&Modifiers::Nln)
                .and_then(|v| match v {
                    ModifierValue::U32(v) => Some(*v),
                    _ => None,
                })
                .unwrap_or(0),
            self.modifiers
                .get(&Modifiers::Timing)
                .and_then(|v| match v {
                    ModifierValue::U32(v) => Some(*v),
                    _ => None,
                })
                .unwrap_or(0),
        );

        // Process each player
        for (i, player) in self.players.iter_mut().enumerate() {
            let mut result = PlayerResult {
                slot: i as u8,
                valid: SCORE_INVALID,
                ..Default::default()
            };

            if let UserSlot::User { user, data, .. } = player {
                let rank = ranks[i];
                let user_total_notes =
                    data.cool as i32 + data.good as i32 + data.bad as i32 + data.miss as i32;

                // Double-check exploit post-submission
                if user_total_notes > max_notes {
                    data.finished = false;
                }

                if data.finished {
                    result.valid = SCORE_VALID;

                    // Update User Stats
                    if rank == 0 {
                        user.info.wins += 1;
                    } else {
                        user.info.losses += 1;
                    }

                    let gems_gained = (data.score / 10000) as u32; // Fixed the TODO from your code
                    user.info.exp += (data.score / 1000) as u64;
                    user.info.o2gems += gems_gained;

                    // Build Broadcast Payload
                    result.score = PlayerScore {
                        cool: data.cool,
                        good: data.good,
                        bad: data.bad,
                        miss: data.miss,
                        max_combo: data.max_combo,
                        jam_combo: data.jam_combo,
                        score: data.score,
                        gem_earned: gems_gained as i16,
                        level: user.level() as u32,
                        position: (rank + 1) as u8,
                        ..Default::default()
                    };

                    #[allow(unused_mut)]
                    let mut score = database::Score {
                        id: 0,
                        user_id: user.id,
                        music_id: self.music_id.songid(),
                        score: data.score as u32,
                        cool: data.cool as u32,
                        good: data.good as u32,
                        bad: data.bad as u32,
                        miss: data.miss as u32,
                        max_combo: data.max_combo as u32,
                        jam_combo: data.jam_combo as u32,
                        skills: skill_ids.clone(),
                        timestamp,
                        ..Default::default()
                    };

                    #[cfg(not(feature = "disable-o2hook2-mod"))]
                    {
                        score.timing = timing;
                        score.rate = rate;
                        score.fln = fln;
                        score.sln = sln;
                        score.nln = nln;
                    }

                    // Prepare DB Score Payload
                    scores_to_save.push(score);

                    // Spawn save task for individual user
                    let user_clone = user.clone();
                    tokio::spawn(async move {
                        if let Err(e) = user_clone.save().await {
                            println!("[Error] Failed to save user {}: {}", user_clone.id, e);
                        }
                    });
                }
            }
            results.push(result);
        }

        // Batch save match scores
        if !scores_to_save.is_empty() {
            let room_id = self.id;
            tokio::spawn(async move {
                let pool = crate::database::get();
                if let Err(e) = pool.submit_scores(&scores_to_save).await {
                    println!("[Error] Failed to save scores for room {}: {}", room_id, e);
                }
            });
        }

        // Broadcast & Reset
        self.broadcast(EventId::GameOnFinish, GameFinishEventArgs { results }, None);
        self.set_status(RoomStatus::Waiting).await;
    }

    pub fn confirm_game_loaded(&mut self, user: u64) {
        if let Some((slot, UserSlot::User { data, .. })) = self.get_user_slot_mut(user) {
            data.loaded = true;

            self.broadcast(
                EventId::GameOnLoadingReady,
                GameOnLoadingReadyEventArgs { slot: slot as u8 },
                None,
            );
        } else {
            println!("User is not in room {}", self.id);
        }
    }

    #[cfg(not(feature = "disable-o2hook2-mod"))]
    pub fn get_modifier_report(&self) -> ModifierReport {
        let rate = match self.modifiers.get(&Modifiers::Rate) {
            Some(ModifierValue::Float(f)) => *f,
            _ => 1.0,
        };

        let fln = match self.modifiers.get(&Modifiers::Fln) {
            Some(ModifierValue::U32(v)) => *v,
            _ => 0,
        };

        let sln = match self.modifiers.get(&Modifiers::Sln) {
            Some(ModifierValue::U32(v)) => *v,
            _ => 0,
        };

        let nln = match self.modifiers.get(&Modifiers::Nln) {
            Some(ModifierValue::U32(v)) => *v,
            _ => 0,
        };

        let timing_bpm = match self.modifiers.get(&Modifiers::Timing) {
            Some(ModifierValue::U32(v)) => *v,
            _ => 0,
        };

        ModifierReport {
            rate,
            fln,
            sln,
            nln,
            timing_bpm,
        }
    }

    #[cfg(not(feature = "disable-o2hook2-mod"))]
    pub async fn set_modifier(&mut self, user_id: u64, modifier: Modifiers, value: u32) {
        if !self.is_host(user_id) {
            println!("User {} is not the host of room {}", user_id, self.id);
            return;
        }

        match modifier {
            Modifiers::Rate => self
                .modifiers
                .insert(Modifiers::Rate, ModifierValue::Float(f32::from_bits(value))),
            Modifiers::Fln => self
                .modifiers
                .insert(Modifiers::Fln, ModifierValue::U32(value)),
            Modifiers::Sln => self
                .modifiers
                .insert(Modifiers::Sln, ModifierValue::U32(value)),
            Modifiers::Nln => self
                .modifiers
                .insert(Modifiers::Nln, ModifierValue::U32(value)),
            Modifiers::Timing => self
                .modifiers
                .insert(Modifiers::Timing, ModifierValue::U32(value)),
            _ => None,
        };

        if modifier == Modifiers::Rate {
            let title = self.title.clone();
            self.set_name(user_id, &title).await;
        }

        self.broadcast(
            EventId::RoomOnModifierChanged,
            RoomOnModifierChangedEventArgs { modifier, value },
            None,
        );
    }

    #[cfg(not(feature = "disable-o2hook2-mod"))]
    pub async fn set_all_modifiers(&mut self, user_id: u64, modifiers: ModifierReport) {
        if !self.is_host(user_id) {
            println!("User {} is not the host of room {}", user_id, self.id);
            return;
        }

        if modifiers.rate != 1.0 {
            let title = self.title.clone();
            self.set_name(user_id, &title).await;
        }

        self.modifiers
            .insert(Modifiers::Rate, ModifierValue::Float(modifiers.rate));
        self.modifiers
            .insert(Modifiers::Fln, ModifierValue::U32(modifiers.fln));
        self.modifiers
            .insert(Modifiers::Sln, ModifierValue::U32(modifiers.sln));
        self.modifiers
            .insert(Modifiers::Nln, ModifierValue::U32(modifiers.nln));
        self.modifiers
            .insert(Modifiers::Timing, ModifierValue::U32(modifiers.timing_bpm));

        self.broadcast(
            EventId::RoomOnModifierChanged,
            RoomOnAllModifiersChangedEventArgs {
                modifiers: self.get_modifier_report(),
            },
            None,
        );
    }

    pub async fn leave_game(&mut self, user: u64) -> bool {
        let (slot, level) =
            if let Some((slot, UserSlot::User { user, .. })) = self.get_user_slot_mut(user) {
                (slot, user.level())
            } else {
                println!("User is not in room {}", self.id);
                return false;
            };

        self.broadcast(
            EventId::GameOnPlayerLeave,
            GameOnPlayerLeaveEventArgs {
                slot: slot as u8,
                level,
            },
            None,
        );

        // If game is active and more than 1 player, count as loss for leaver and remove from room.
        // Otherwise, the game expect to keep the lobby if just solo.
        if self.player_count() > 1 && self.status == RoomStatus::Playing {
            if let Some((_, UserSlot::User { user, .. })) = self.get_user_slot_mut(user) {
                user.info.losses += 1;

                user.save()
                    .await
                    .expect("Failed to save user data after leaving game");
            }

            self.remove_user(user).await;

            // Removed from the room.
            return true;
        } else {
            self.set_status(RoomStatus::Waiting).await;

            return false;
        }
    }

    pub async fn chat(&mut self, user: u64, message: &str) {
        let nickname = if let Some((_, UserSlot::User { user, .. })) = self.get_user_slot_mut(user)
        {
            to_cstring(&user.nickname())
        } else {
            println!("User is not in room {}", self.id);
            return;
        };

        self.broadcast(
            EventId::RoomOnChat,
            RoomOnChatEventArgs {
                author: nickname,
                message: to_cstring(message),
            },
            None,
        );
    }

    pub async fn toggle_slot(&mut self, user_id: u64, slot: usize) {
        if !self.is_host(user_id) {
            println!("User {} is not the host of room {}", user_id, self.id);
            return;
        }

        if slot >= self.players.len() {
            println!("Invalid slot index {} for room {}", slot, self.id);
            return;
        }

        let status = match &mut self.players[slot] {
            UserSlot::Available => {
                self.players[slot] = UserSlot::Locked;

                PositionStatus::Locked
            }
            UserSlot::Locked => {
                self.players[slot] = UserSlot::Available;

                PositionStatus::Occupied
            }
            UserSlot::User { .. } => {
                let slot = std::mem::replace(&mut self.players[slot], UserSlot::Available);

                // User kicked
                let _ = self.channel_handle.send::<()>(ChannelCommand::Kicked {
                    user_id: match slot {
                        UserSlot::User { user, .. } => user.id,
                        _ => unreachable!(),
                    },
                });

                PositionStatus::Empty
            }
        };

        self.broadcast_channel(
            EventId::ListRoomOnRoomPlayerCountChanged,
            ListRoomChangeRoomMaxPlayerEventArgs {
                id: self.id,
                max_player: self.max_players() as u8,
                current_player: self.player_count() as u8,
                premium: 0,
            },
            None,
        )
        .await;

        // Broadcast the change to all players
        self.broadcast(
            EventId::RoomOnSlotChanged,
            RoomOnSlotChangedEventArgs {
                slot: slot as u8,
                status,
            },
            None,
        );
    }

    pub async fn set_music_state(&mut self, user: u64, state_: PlayingState) {
        let Some((slot, UserSlot::User { state, .. })) = self.get_user_slot_mut(user) else {
            println!("User is not in room {}", self.id);
            return;
        };

        *state = state_;

        self.broadcast(
            EventId::RoomOnMusicStateChanged,
            RoomOnMusicStateChangedEventArgs {
                slot: slot as u8,
                state: state_,
            },
            None,
        );
    }

    pub fn broadcast<T: IEventData + 'static>(
        &self,
        id: EventId,
        event: T,
        exception: Option<&User>,
    ) {
        let boxed = Arc::new(event) as Arc<dyn IEventData>;

        for player in &self.players {
            if let UserSlot::User {
                user: player_user, ..
            } = player
            {
                if let Some(exception_user) = exception {
                    if player_user == exception_user {
                        continue; // Skip the exception user
                    }
                }

                let _ = player_user.get_sender().send((id, boxed.clone()));
            }
        }
    }

    pub async fn broadcast_channel<T: IEventData + 'static>(
        &self,
        id: EventId,
        event: T,
        exception: Option<&User>,
    ) {
        self.channel_handle
            .send::<()>(ChannelCommand::BroadcastEvent {
                id,
                event: Arc::new(event) as Arc<dyn IEventData>,
                exception: exception.map(|u| u.id),
            })
            .await
            .expect("Failed to broadcast event to channel");
    }
}

pub fn gen_random_seed() -> u32 {
    let mut seed = [0u8; 4];
    getrandom::fill(&mut seed).expect("Failed to generate random seed");

    u32::from_le_bytes(seed)
}

pub fn to_cstring(s: &str) -> std::ffi::CString {
    std::ffi::CString::new(s).expect("Failed to convert string to CString")
}
