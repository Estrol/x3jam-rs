use std::sync::Arc;

use crate::gateway::{
    commands::{EventId, ResponseId},
    events::connection::DisconnectEventArgs,
};

#[gateway_derive::route(RequestId::Disconnect)]
async fn handle_disconnect(client: &mut super::Client, _packet: &mut super::Packet) {
    let Some(sender) = client.sender.as_ref() else {
        panic!("Attempted to send disconnect event for client without sender");
    };

    sender
        .send((EventId::Disconnect, Arc::new(DisconnectEventArgs)))
        .expect("Failed to send disconnect event");
}

#[gateway_derive::route(RequestId::GatewayPing)]
async fn handle_ping(client: &mut super::Client, _packet: &mut super::Packet) {
    client
        .send_packet(ResponseId::GatewayPing, &())
        .await
        .expect("Failed to send ping response");
}
