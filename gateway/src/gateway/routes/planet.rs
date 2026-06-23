use std::sync::Arc;

use crate::gateway::commands::ResponseId;

#[gateway_derive::route(RequestId::PlanetGetChannels)]
async fn handle_channel(client: &mut super::Client, _packet: &mut super::Packet) {
    #[derive(encoder::StructSerializer)]
    struct ChannelEntry {
        server_id: u16,
        channel_id: u16,
        max_players: u32,
        current_players: u32,
        is_active: bool,
    }

    let channels = crate::gateway::GET_CHANNELS().await;
    let mut entries = Vec::new();

    for handle in channels.iter() {
        let channel = handle.0.lock().await;

        entries.push(ChannelEntry {
            server_id: handle.1 as u16,
            channel_id: handle.2 as u16,
            max_players: channel.max_users as u32,
            current_players: channel.users.len() as u32,
            is_active: true,
        });
    }

    client
        .send_packet(ResponseId::PlanetGetChannels, &entries)
        .await
        .expect("Failed to send channel list response");
}

#[gateway_derive::route(RequestId::PlanetEnterChannel)]
async fn handle_enter_channel(client: &mut super::Client, packet: &mut super::Packet) {
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

    match super::parse_request::<EnterChannelRequest>(&packet.body) {
        Ok(request) => {
            let mut response = EnterChannelResponse {
                result: 0,        // Success
                is_restricted: 0, // Not restricted
            };

            let (success, err) = {
                let mut success = false;
                let mut err = false;

                let channel_weak = {
                    let channels = crate::gateway::GET_CHANNELS().await;
                    let mut channel_weak = None;

                    if let Some(user) = client.user() {
                        for handle in channels.iter() {
                            if handle.1 == request.server_id as u32
                                && handle.2 == request.channel_id as u32
                            {
                                let mut channel = handle.0.lock().await;

                                // Step 1: Add user to channel
                                success = channel.add_user(user).await;

                                // Step 2: Set client's channel
                                if success {
                                    channel_weak = Some(Arc::downgrade(&handle.0));
                                    break;
                                }
                            }
                        }
                    } else {
                        err = true;
                    }

                    channel_weak
                };

                client.channel = channel_weak;

                (success, err)
            };

            if !success {
                response.result = 1; // Error

                if err {
                    response.is_restricted = 1; // Restricted (not logged in)
                }
            }

            client
                .send_packet(ResponseId::PlanetEnterChannel, &response)
                .await
                .expect("Failed to send enter channel response");
        }
        Err(e) => {
            println!("[Error] Failed to parse enter channel request: {}", e);

            let response = EnterChannelResponse {
                result: 1, // Error
                is_restricted: 1,
            };

            client
                .send_packet(ResponseId::PlanetEnterChannel, &response)
                .await
                .expect("Failed to send enter channel error response");
        }
    }
}

#[gateway_derive::route(RequestId::PlanetLeaveChannel)]
async fn handle_leave_channel(client: &mut super::Client, _packet: &mut super::Packet) {
    if let Some(channel) = client.channel() {
        let mut channel = channel.lock().await;

        if let Some(user) = client.user() {
            channel.remove_user(user).await;
        }
    }

    client.channel = None;

    client
        .send_packet(ResponseId::PlanetLeaveChannel, &0)
        .await
        .expect("Failed to send leave channel response");
}
