use crate::{
    gateway::commands::ResponseId,
    room::{GameEventType, RoomCommand},
};

#[derive(Clone, encoder::StructDeserializer)]
pub struct ScoreSubmitRequest {
    pub cool: i16,
    pub good: i16,
    pub bad: i16,
    pub miss: i16,
    pub max_combo: i16,
    pub jam_combo: i16,
    pub passed: i16,
    pub score: i32,
    pub padding: u8,
    pub arrangement: [u8; 7],
    pub event_count: u32,
    pub hp_graph: [u32; 500],
    pub song_rate: f32,
    pub timing_bpm: u32,
    pub fln: u32,
    pub sln: u32,
    pub nln: u32,
}

impl Default for ScoreSubmitRequest {
    fn default() -> Self {
        Self {
            cool: 0,
            good: 0,
            bad: 0,
            miss: 0,
            max_combo: 0,
            jam_combo: 0,
            passed: 0,
            score: 0,
            arrangement: [0; 7],
            event_count: 0,
            padding: 0,
            hp_graph: [0; 500],
            song_rate: 0.0,
            timing_bpm: 0,
            fln: 0,
            sln: 0,
            nln: 0,
        }
    }
}

#[derive(encoder::StructSerializer)]
pub struct ScoreSubmitResponse {
    pub slot: u8,
    pub success: bool,
}

#[gateway_derive::route(RequestId::SubmitScore)]
pub async fn handle_submit_score(client: &mut crate::gateway::Client, packet: &ScoreSubmitRequest) {
    let Some((user_id, room)) = client.room() else {
        println!("Received SubmitScore request but client is not in a room");
        return;
    };

    let Ok(result) = room
        .send::<ScoreSubmitResponse>(RoomCommand::SubmitScore {
            user_id,
            score_request: packet.clone(),
        })
        .await
    else {
        println!("Failed to send SubmitScore command to room");
        return;
    };

    client
        .send_packet(ResponseId::SubmitScore, &result)
        .await
        .expect("Failed to send SubmitScore response");
}

#[derive(Debug, Clone, encoder::StructDeserializer)]
pub struct GameEventPingRequest {
    pub r#type: GameEventType, // 2
    pub value: u16,            // 4
    pub sequence: u32,         // 8
    pub score: u32,            // 10
}

#[gateway_derive::route(RequestId::GameNoteEvent)]
pub async fn handle_game_on_note_event(
    client: &mut crate::gateway::Client,
    packet: &GameEventPingRequest,
) {
    let Some((user_id, room)) = client.room() else {
        println!("Received GameOnNoteEvent request but client is not in a room");
        return;
    };

    let _ = room
        .send::<()>(RoomCommand::GameEvent {
            user_id,
            event: packet.clone(),
        })
        .await;
}

#[gateway_derive::route(RequestId::GameConfirmLoaded)]
pub async fn handle_game_confirm_loaded(client: &mut crate::gateway::Client, _packet: &()) {
    let Some((user_id, room)) = client.room() else {
        println!("Received GameConfirmLoaded request but client is not in a room");
        return;
    };

    let _ = room
        .send::<()>(RoomCommand::ConfirmGameLoaded { user_id })
        .await;
}

#[gateway_derive::route(RequestId::GameLeave)]
pub async fn handle_game_leave(client: &mut crate::gateway::Client, _packet: &()) {
    let Some((user_id, room)) = client.room() else {
        println!("Client is not in a room");
        return;
    };

    let Ok(leaving_room) = room.send::<bool>(RoomCommand::LeaveGame { user_id }).await else {
        println!("Failed to send leave game command to room");
        return;
    };

    if leaving_room {
        client.room_handle = None;
    }
}
