use std::sync::Arc;

use crate::gateway::{Client, commands::EventId};

pub mod connection;
pub mod game;
pub mod listroom;
pub mod room;

pub trait IEventData: std::any::Any + Send + Sync {}

pub fn downcast<T: IEventData + 'static>(data: &dyn IEventData) -> Option<&T> {
    let any = data as &dyn std::any::Any;
    any.downcast_ref::<T>()
}

pub struct Event {
    id: EventId,
    handler: for<'a> fn(
        &'a mut Client,
        &'a dyn IEventData,
    )
        -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>>,
}

inventory::collect!(Event);

pub async fn handle_event(client: &mut Client, event_id: EventId, data: Arc<dyn IEventData>) {
    client.begin().expect("Failed to begin event batch");

    if let Some(event) = get_event(event_id) {
        (event.handler)(client, &*data).await;
    } else {
        log::info!("Received unhandled event: ID={:?}", event_id);
    }

    client.end().await.expect("Failed to end event batch");
}

pub fn get_event(event_id: EventId) -> Option<&'static Event> {
    for event in inventory::iter::<Event> {
        if event.id == event_id {
            return Some(event);
        }
    }

    None
}
