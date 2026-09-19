#[derive(gateway_derive::Event)]
pub struct DisconnectEventArgs;

pub async fn handle_event_disconnect(client: &mut super::Client, _data: &dyn super::IEventData) {
    log::info!("Client {} disconnected", client.id);
}
