use crate::gateway::{
    commands::ResponseId,
    room::GameEventType,
    routes::Packet,
};

#[derive(encoder::StructDeserializer, Default)]
pub struct ScoreSubmitRequest {
    pub cool: i16,
    pub good: i16,
    pub bad: i16,
    pub miss: i16,
    pub max_combo: i16,
    pub jam_combo: i16,
    pub passed: i16,
    pub score: i32,
}

#[derive(encoder::StructSerializer)]
pub struct ScoreSubmitResponse {
    slot: u8,
    success: bool,
}

#[gateway_derive::route(RequestId::SubmitScore)]
pub async fn handle_submit_score(client: &mut crate::gateway::Client, packet: &mut Packet) {
    let Some(room) = client.room() else {
        println!("Received SubmitScore request but client is not in a room");
        return;
    };

    let Some(user) = client.user() else {
        println!("Received SubmitScore request but client is not authenticated");
        return;
    };

    match super::parse_request::<ScoreSubmitRequest>(&packet.body) {
        Ok(request) => {
            let response = {
                let mut room = room.lock().await;

                let (success, slot) = room.on_game_score_submit(&user, request).await;

                ScoreSubmitResponse {
                    slot: slot as u8,
                    success, // False = forced kicked from game, true = score accepted
                }
            };

            client
                .send_packet(ResponseId::SubmitScore, &response)
                .await
                .expect("Failed to send SubmitScore response");
        }
        Err(e) => {
            println!("Failed to parse SubmitScore request: {:?}", e);
        }
    }
}

#[derive(Debug, encoder::StructDeserializer)]
pub struct GameEventPingRequest {
    pub r#type: GameEventType, // 2
    pub value: u16,            // 4
    pub sequence: u32,         // 8
    pub score: u32,            // 10
}

#[gateway_derive::route(RequestId::GameNoteEvent)]
pub async fn handle_game_on_note_event(client: &mut crate::gateway::Client, packet: &mut Packet) {
    let Some(room) = client.room() else {
        println!("Received GameOnNoteEvent request but client is not in a room");
        return;
    };

    let Some(user) = client.user() else {
        println!("Received GameOnNoteEvent request but client is not authenticated");
        return;
    };

    match super::parse_request::<GameEventPingRequest>(&packet.body) {
        Ok(request) => {
            let mut room = room.lock().await;

            room.on_game_event(&user, request).await;
        }
        Err(e) => {
            println!("Failed to parse GameOnNoteEvent request: {:?}", e);
        }
    }
}

#[gateway_derive::route(RequestId::GameConfirmLoaded)]
pub async fn handle_game_confirm_loaded(client: &mut crate::gateway::Client, _packet: &mut Packet) {
    let Some(room) = client.room() else {
        println!("Received GameConfirmLoaded request but client is not in a room");
        return;
    };

    let Some(user) = client.user() else {
        println!("Received GameConfirmLoaded request but client is not authenticated");
        return;
    };

    room.lock().await.confirm_game_loaded(&user);
}

#[gateway_derive::route(RequestId::GameLeave)]
pub async fn handle_game_leave(client: &mut crate::gateway::Client, _packet: &mut Packet) {
    let Some(room) = client.room() else {
        println!("Client is not in a room");
        return;
    };

    let mut room = room.lock().await;

    let Some(user) = client.user() else {
        println!("Client is not authenticated");
        return;
    };

    room.leave_game(user).await;
}
