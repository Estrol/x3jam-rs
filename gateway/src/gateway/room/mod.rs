use std::{
    cmp::Reverse, collections::HashMap, sync::{Arc, Weak}
};

use tokio::sync::Mutex;

use crate::gateway::{
    commands::EventId::{self},
    events::{
        IEventData,
        game::{
            ArrayResult, GameOnLoadingReadyEventArgs, GameOnNoteEventEventArgs, GameOnPlayerLeaveEventArgs, PlayerResult, PlayerScore, SCORE_INVALID, SCORE_VALID
        },
        listroom::{
            ListRoomChangeRoomMusicIdEventArgs, ListRoomChangeRoomNameEventArgs, ListRoomChangeRoomSkillEventArgs, ListRoomChangeRoomStatusEventArgs
        },
        room::{
            RoomOnArenaChangedEventArgs, RoomOnChatEventArgs, RoomOnGameStartEventArgs, RoomOnModifierChangedEventArgs, RoomOnMusicIdChangedEventArgs, RoomOnNameChangedEventArgs, RoomOnPlayerEnterEventArgs, RoomOnPlayerLeaveEventArgs, RoomOnReadyEventArgs, RoomOnSkillChangedEventArgs, RoomOnTeamChangedEventArgs
        },
    },
    routes::{
        game::{GameEventPingRequest, ScoreSubmitRequest},
        room::GameStartResult,
    },
    user::User,
};

pub mod arena;
pub mod difficulty;
pub mod eventtype;
pub mod mode;
pub mod music;
pub mod skill;
pub mod speed;
pub mod status;
pub mod team;
pub mod modifier;

pub use arena::RoomArena;
pub use difficulty::RoomDifficulty;
pub use eventtype::GameEventType;
pub use mode::RoomMode;
pub use music::{MusicId, MusicIdEntry};
pub use skill::SkillId;
pub use speed::RoomSpeed;
pub use status::RoomStatus;
pub use team::TeamId;
pub use modifier::{Modifiers, ModifierReport};

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
    pub parent: Weak<Mutex<super::channel::Channel>>,

    pub title: String,
    pub password: Option<String>,
    pub color: TeamId,

    pub mode: RoomMode,
    pub difficulty: RoomDifficulty,
    pub arena: RoomArena,
    pub arena_random: bool,
    pub speed: RoomSpeed,
    pub music_id: MusicId,
    pub status: RoomStatus,

    pub skill_slot: Vec<SkillId>,
    pub skill_seed: u32,
    pub players: [UserSlot; 8],

    pub min_level: u8,
    pub max_level: u8,
    pub modifiers: HashMap<Modifiers, ModifierValue>,

    pub join_pos_counter: u64, // Counter to assign join positions to players
}

impl Room {
    pub fn new(
        channel: Weak<Mutex<super::channel::Channel>>,
        user: &User,

        id: u32,
        title: String,
        password: Option<String>,
        mode: RoomMode,
        min_level: u8,
        max_level: u8,
    ) -> Self {
        let mut room = Room {
            id,
            parent: channel,
            title,
            password,
            color: TeamId::Red,
            mode,
            difficulty: RoomDifficulty::Hard,
            arena: RoomArena::ARENA1,
            arena_random: false,
            speed: RoomSpeed::Speed10,
            music_id: MusicId(0),
            status: RoomStatus::Waiting,
            skill_slot: Vec::new(),
            skill_seed: gen_random_seed(),
            max_level,
            min_level,
            modifiers: HashMap::from_iter(
                [
                    (Modifiers::Rate, ModifierValue::Float(1.0)),
                    (Modifiers::Fln, ModifierValue::U32(0)),
                    (Modifiers::Sln, ModifierValue::U32(0)),
                    (Modifiers::Nln, ModifierValue::U32(0)),
                    (Modifiers::Timing, ModifierValue::U32(0)),
                ]
            ),
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
            join_pos: 0,
        };

        room
    }

    pub fn get_user_slot_index(&self, user: &User) -> Option<usize> {
        self.players.iter().position(
            |p| matches!(p, UserSlot::User { user: player_user, .. } if player_user.id == user.id),
        )
    }

    pub fn get_user_slot(&self, user: &User) -> Option<(usize, &UserSlot)> {
        self.players.iter().enumerate().find_map(|(i, p)| {
            if let UserSlot::User { user: player_user, .. } = p {
                if player_user.id == user.id {
                    return Some((i, p));
                }
            }
            None
        })
    }

    pub fn get_user_slot_mut(&mut self, user: &User) -> Option<(usize, &mut UserSlot)> {
        self.players.iter_mut().enumerate().find_map(|(i, p)| {
            if let UserSlot::User { user: player_user, .. } = p {
                if player_user.id == user.id {
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

    pub fn add_player(&mut self, user: &User) -> Option<(u8, TeamId)> {
        let slot = self
            .players
            .iter()
            .position(|p| matches!(p, UserSlot::Available));

        let Some(slot) = slot else {
            println!("No available slots in room {}", self.id);
            return None;
        };

        let color = self.next_color();
        let join_pos = self.get_next_counter();

        self.players[slot] = UserSlot::User {
            id: user.id,
            host: false,
            ready: false,
            team: color,
            data: GameData::default(),
            user: user.clone(),
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

        Some((slot as u8, color))
    }

    pub async fn remove_client(&mut self, user: &User) -> Option<usize> {
        if let Some(slot) = self.get_user_slot_index(user) {
            if self.player_count() > 1 && self.status == RoomStatus::Playing {
                self.on_game_score_submit(user, ScoreSubmitRequest::default()).await;
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
                    Some(user),
                );

                println!(
                    "User {} left room {} (slot {}), new host is slot {}",
                    user.id,
                    self.id,
                    slot,
                    host
                );
            } else {
                println!(
                    "User {} left room {} (slot {}), room is now empty",
                    user.id, self.id, slot
                );
            }

            Some(self.player_count())
        } else {
            println!("User is not in room {}", self.id);
            None
        }
    }

    pub fn is_host(&self, user: &User) -> bool {
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
        all_active_players!(self.players, UserSlot::User { ready, .. } => *ready)
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

    pub async fn set_ready(&mut self, user: &User) {
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

    pub async fn game_start(&mut self) {
        self.skill_seed = gen_random_seed();

        // Elegantly reset game data using standard Default trait
        for player in &mut self.players {
            if let UserSlot::User { data, .. } = player {
                *data = GameData::default();
            }
        }

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
    }

    pub async fn set_name(&mut self, name: &str) {
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
        ).await;
    }

    pub fn get_name_rate(&self) -> String {
        let title = if self.modifiers.contains_key(&Modifiers::Rate) && let ModifierValue::Float(rate) = self.modifiers[&Modifiers::Rate] && rate != 1.0 {
            format!("[{:.2}x] {}", rate, self.title)
        } else {
            self.title.clone()
        };

        title
    }

    pub async fn set_song_id(
        &mut self,
        music_id: MusicId,
        difficulty: RoomDifficulty,
        speed: RoomSpeed,
    ) {
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

    pub fn set_arena(&mut self, arena: RoomArena, random: bool) {
        self.arena = arena;
        self.arena_random = random;

        self.broadcast(
            EventId::RoomOnArenaChanged,
            RoomOnArenaChangedEventArgs { arena, random },
            None,
        );
    }

    pub async fn set_ring(&mut self, ring: Vec<SkillId>) {
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

    pub async fn set_team(&mut self, user: &User, team: TeamId) {
        if let Some((slot, UserSlot::User { team: player_team, .. })) = self.get_user_slot_mut(user) {
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

    pub async fn on_game_event(&mut self, user: &User, event: GameEventPingRequest) {
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
        user: &User,
        result: ScoreSubmitRequest,
    ) -> (bool, usize) {
        let max_notes = self.fetch_max_notes().await;

        // Early return: Invert the check to avoid deep nesting
        let slot = match self.get_user_slot_mut(user) {
            Some((slot, UserSlot::User { data, .. })) => {
                if data.finished {
                    return (true, slot);
                }

                let user_total_notes = result.cool as i32 + result.good as i32 + result.bad as i32 + result.miss as i32;
                if user_total_notes > max_notes {
                    return (false, slot); // Reject: likely score exploit
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
                println!("[Warn] User {} is not in room {}", user.id, self.id);
                return (false, 0);
            }
        };

        // Orchestrate match finish if applicable
        if self.is_everyone_finished() {
            self.process_match_results(max_notes).await;
        }

        (true, slot)
    }

    async fn fetch_max_notes(&self) -> i32 {
        let channel_arc = match self.parent.upgrade() {
            Some(c) => c,
            None => return 0,
        };

        let channel = channel_arc.lock().await;
        
        channel.list.iter()
            .find(|ojn| ojn.songid == self.music_id.0 as i32)
            .map(|ojn| match self.difficulty {
                RoomDifficulty::Easy => ojn.note_count[0],
                RoomDifficulty::Normal => ojn.note_count[1],
                RoomDifficulty::Hard => ojn.note_count[2],
            })
            .unwrap_or(0)
    }

    async fn process_match_results(&mut self, max_notes: i32) {
        // Rank players efficiently (sort descending)
        let mut positions: Vec<(usize, i32)> = self.players.iter().enumerate()
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

        // Process each player
        for (i, player) in self.players.iter_mut().enumerate() {
            let mut result = PlayerResult {
                slot: i as u8,
                valid: SCORE_INVALID,
                ..Default::default()
            };

            if let UserSlot::User { user, data, .. } = player {
                let rank = ranks[i];
                let user_total_notes = data.cool as i32 + data.good as i32 + data.bad as i32 + data.miss as i32;
                
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

                    // Prepare DB Score Payload
                    scores_to_save.push(database::Score {
                        id: 0,
                        user_id: user.id,
                        music_id: self.music_id.0 as u32,
                        score: data.score as u32,
                        cool: data.cool as u32,
                        good: data.good as u32,
                        bad: data.bad as u32,
                        miss: data.miss as u32,
                        max_combo: data.max_combo as u32,
                        jam_combo: data.jam_combo as u32,
                        rate: 1.0,
                        skills: skill_ids.clone(),
                        timestamp,
                    });

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
                let pool = crate::gateway::GET_DATABASE().await;
                if let Err(e) = pool.submit_scores(&scores_to_save).await {
                    println!("[Error] Failed to save scores for room {}: {}", room_id, e);
                }
            });
        }

        // Broadcast & Reset
        self.broadcast(EventId::GameOnFinish, ArrayResult { results }, None);
        self.set_status(RoomStatus::Waiting).await;
    }

    pub fn confirm_game_loaded(&mut self, user: &User) {
        if let Some((slot, UserSlot::User { data, .. })) = self.get_user_slot_mut(user) {
            data.loaded = true;

            self.broadcast(
                EventId::GameOnLoadingReady,
                GameOnLoadingReadyEventArgs {
                    slot: slot as u8,
                },
                None,
            );
        } else {
            println!("User is not in room {}", self.id);
        }
    }

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

    pub async fn set_modifier(&mut self, modifier: Modifiers, value: u32) {
        match modifier {
            Modifiers::Rate => self.modifiers.insert(Modifiers::Rate, ModifierValue::Float(f32::from_bits(value))),
            Modifiers::Fln => self.modifiers.insert(Modifiers::Fln, ModifierValue::U32(value)),
            Modifiers::Sln => self.modifiers.insert(Modifiers::Sln, ModifierValue::U32(value)),
            Modifiers::Nln => self.modifiers.insert(Modifiers::Nln, ModifierValue::U32(value)),
            Modifiers::Timing => self.modifiers.insert(Modifiers::Timing, ModifierValue::U32(value)),
            _ => None
        };

        if modifier == Modifiers::Rate {
            let title = self.title.clone();
            self.set_name(&title).await;
        }

        self.broadcast(
            EventId::RoomOnModifierChanged,
            RoomOnModifierChangedEventArgs { modifier, value },
            None,
        );
    }

    pub async fn set_all_modifiers(&mut self, modifiers: ModifierReport) {
        if modifiers.rate != 1.0 {
             let title = self.title.clone();
             self.set_name(&title).await;
        }

        self.modifiers.insert(Modifiers::Rate, ModifierValue::Float(modifiers.rate));
        self.modifiers.insert(Modifiers::Fln, ModifierValue::U32(modifiers.fln));
        self.modifiers.insert(Modifiers::Sln, ModifierValue::U32(modifiers.sln));
        self.modifiers.insert(Modifiers::Nln, ModifierValue::U32(modifiers.nln));
        self.modifiers.insert(Modifiers::Timing, ModifierValue::U32(modifiers.timing_bpm));

        self.broadcast(
            EventId::RoomOnModifierChanged,
            RoomOnModifierChangedEventArgs { 
                modifier: Modifiers::Rate, 
                value: (modifiers.rate * 100.0) as u32,
            },
            None,
        );
    }

    pub async fn leave_game(&mut self, user: &User) {
        let Some(slot_idx) = self.get_user_slot_index(user) else {
            println!("User is not in room {}", self.id);
            return;
        };

        self.broadcast(
            EventId::GameOnPlayerLeave,
            GameOnPlayerLeaveEventArgs {
                slot: slot_idx as u8,
                level: user.level(),
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

            self.remove_client(user).await;
        } else {
            self.set_status(RoomStatus::Waiting).await;
        }
    }

    pub async fn on_chat(&mut self, user: &User, message: &str) {
        if let Some((_, UserSlot::User { .. })) = self.get_user_slot_mut(user) {
            self.broadcast(
                EventId::RoomOnChat,
                RoomOnChatEventArgs {
                    author: to_cstring(&user.nickname()),
                    message: to_cstring(message),
                },
                None,
            );
        } else {
            println!("User is not in room {}", self.id);
        }
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

                let _ = player_user
                    .get_sender()
                    .send((id, boxed.clone()));
            }
        }
    }

    pub async fn broadcast_channel<T: IEventData + 'static>(
        &self,
        id: EventId,
        event: T,
        exception: Option<&User>,
    ) {
        if let Some(channel) = self.parent.upgrade() {
            let mut channel = channel.lock().await;

            channel.broadcast(id, event, exception);
        }
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
