use crate::{channel::ChannelCommand, gateway::commands::ResponseId};

#[gateway_derive::route(RequestId::PlanetGetChannels)]
async fn handle_channel(client: &mut super::Client, _packet: &()) {
    #[derive(encoder::StructSerializer)]
    struct ChannelEntry {
        server_id: u16,
        channel_id: u16,
        max_players: u32,
        current_players: u32,
        is_active: bool,
    }

    let channels = crate::gateway::GET_CHANNELS();
    let mut entries = Vec::new();

    for handle in channels.iter() {
        entries.push(ChannelEntry {
            server_id: handle.region() as u16,
            channel_id: handle.id() as u16,
            max_players: handle.max_users() as u32,
            current_players: handle.current_users() as u32,
            is_active: true,
        });
    }

    client
        .send_packet(ResponseId::PlanetGetChannels, &entries)
        .await
        .expect("Failed to send channel list response");
}

#[derive(encoder::StructDeserializer)]
struct EnterChannelRequest {
    server_id: u16,
    channel_id: u16,
}

#[derive(encoder::StructSerializer)]
struct EnterChannelResponse {
    result: i32,
    is_restricted: u32,
}

#[gateway_derive::route(RequestId::PlanetEnterChannel)]
async fn handle_enter_channel(client: &mut super::Client, request: &EnterChannelRequest) {
    if !client.version_checked {
        let version_error_msg: &std::ffi::CStr =
            c"You need use latest O2Hook2.dll to able login this server";

        client
            .send_packet(0xABCE as u16, &version_error_msg)
            .await
            .expect("Failed to send re-auth completion packet");

        return;
    }

    let channels = crate::gateway::GET_CHANNELS();

    for handle in channels.iter() {
        if handle.region() == request.server_id as u32 && handle.id() == request.channel_id as u32 {
            let Some(user) = client.user() else {
                println!("Client is not logged in");
                return;
            };

            let Ok(success) = handle
                .send(ChannelCommand::Connect { user: user.clone() })
                .await
            else {
                println!("Failed to send connect command to channel");
                return;
            };

            client.set_channel_handle(handle.make_weak());

            let response = EnterChannelResponse {
                result: if success { 0 } else { 1 },
                is_restricted: 0,
            };

            client
                .send_packet(ResponseId::PlanetEnterChannel, &response)
                .await
                .expect("Failed to send enter channel response");

            return;
        }
    }
}

#[gateway_derive::route(RequestId::PlanetLeaveChannel)]
async fn handle_leave_channel(client: &mut super::Client, _packet: &()) {
    let Some((user_id, ch)) = client.channel() else {
        println!("Client is not in a channel");
        return;
    };

    let Ok(success) = ch
        .send::<bool>(ChannelCommand::Disconnect { user_id })
        .await
    else {
        println!("Failed to send disconnect command to channel");
        return;
    };

    client.channel_handle = None;
    let ret_value = if success { 0 } else { 1 };

    client
        .send_packet(ResponseId::PlanetLeaveChannel, &ret_value)
        .await
        .expect("Failed to send leave channel response");
}
