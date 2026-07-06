use crate::{gateway::commands::EventId, room::GameEventType};

#[derive(gateway_derive::Event, encoder::StructSerializer, Debug)]
pub struct GameOnNoteEventEventArgs {
    pub slot: u8,
    pub r#type: GameEventType,
    pub value: u16,
    pub ranks: [u8; 8],
}

#[gateway_derive::event(EventId::GameOnNoteEvent)]
pub async fn handle_game_on_note_event(
    client: &mut crate::gateway::Client,
    data: &GameOnNoteEventEventArgs,
) {
    client
        .send_packet(EventId::GameOnNoteEvent, data)
        .await
        .expect("Failed to send GameOnNoteEvent");
}

pub const SCORE_VALID: u32 = 1;
pub const SCORE_INVALID: u32 = 0;

#[derive(encoder::StructSerializer, Clone, Debug, Default)]
pub struct PlayerScore {
    pub cool: i16,
    pub good: i16,
    pub bad: i16,
    pub miss: i16,
    pub max_combo: i16,
    pub jam_combo: i16,
    pub score: i32,
    pub gem_earned: i16,
    pub level: u32,
    pub padding: u32,
    pub position: u8,
}

#[derive(gateway_derive::Event, Clone, Default)]
pub struct PlayerResult {
    pub slot: u8,
    pub valid: u32,
    pub score: PlayerScore,
}

impl encoder::StructEncodeImpl for PlayerResult {
    fn impl_encode(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        self.slot.impl_encode(writer)?;
        self.valid.impl_encode(writer)?;
        self.score.impl_encode(writer)?;

        if self.valid == SCORE_VALID {
            // Additional 1 byte trailing when valid is == 1
            0u8.impl_encode(writer)?;
        }

        Ok(())
    }
}

#[derive(gateway_derive::Event, encoder::StructSerializer)]
pub struct GameFinishEventArgs {
    pub results: Vec<PlayerResult>,
}

#[gateway_derive::event(EventId::GameOnFinish)]
pub async fn handle_game_on_finish(
    client: &mut crate::gateway::Client,
    data: &GameFinishEventArgs,
) {
    client
        .send_packet(EventId::GameOnFinish, data)
        .await
        .expect("Failed to send GameOnFinish");
}

#[derive(gateway_derive::Event, encoder::StructSerializer)]
pub struct GameOnLoadingReadyEventArgs {
    pub slot: u8,
}

#[gateway_derive::event(EventId::GameOnLoadingReady)]
pub async fn handle_game_on_loading_ready(
    client: &mut crate::gateway::Client,
    data: &GameOnLoadingReadyEventArgs,
) {
    client
        .send_packet(EventId::GameOnLoadingReady, data)
        .await
        .expect("Failed to send GameOnLoadingReady");
}

#[derive(gateway_derive::Event, encoder::StructSerializer)]
pub struct GameOnPlayerLeaveEventArgs {
    pub slot: u8,
    pub level: u32,
}

#[gateway_derive::event(EventId::GameOnPlayerLeave)]
pub async fn handle_game_on_player_leave(
    client: &mut crate::gateway::Client,
    data: &GameOnPlayerLeaveEventArgs,
) {
    client
        .send_packet(EventId::GameOnPlayerLeave, data)
        .await
        .expect("Failed to send GameOnPlayerLeave");
}
