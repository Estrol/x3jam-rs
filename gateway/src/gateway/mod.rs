pub mod cbc_decrypt;
pub mod commands;
pub mod events;
pub mod ojnlist;
pub mod routes;
pub mod stateful;

pub mod client;
// pub mod itemlist;

use std::sync::{Arc, OnceLock};

use futures::FutureExt as _;
use tcpserver::{IClient, Server};

use crate::{
    channel::{ChannelCommand, ChannelHandle},
    gateway::{commands::EventId, events::IEventData},
    user::User,
};
pub use client::Client;

const HTTP_HEADERS: &[&[u8]] = &[
    b"GET", b"POST", b"PUT", b"DELETE", b"HEAD", b"OPTIONS", b"CONNECT", b"TRACE", b"PATCH",
];

pub fn is_http_request(data: &[u8]) -> bool {
    for header in HTTP_HEADERS {
        if data.starts_with(header) {
            return true;
        }
    }
    false
}

lazy_static::lazy_static! {
    static ref CHANNELS: OnceLock<Vec<ChannelHandle>> = OnceLock::new();
}

#[allow(non_snake_case)]
pub fn GET_CHANNELS() -> &'static Vec<ChannelHandle> {
    CHANNELS.get().expect("Channels not initialized")
}

pub fn setup_channels(channels: Vec<ChannelHandle>) {
    for channel in &channels {
        println!(
            "Starting channel {} with max users {}",
            channel.id(),
            channel.max_users()
        );
    }

    CHANNELS.set(channels).expect("Failed to set channels");
}

async fn process(client: &mut Client) {
    let (sender, receiver) =
        tokio::sync::mpsc::unbounded_channel::<(EventId, Arc<dyn IEventData>)>();

    client.sender = Some(sender.clone());

    if std::panic::AssertUnwindSafe(process_guard(client, receiver))
        .catch_unwind()
        .await
        .is_err()
    {
        println!("Client {} panicked during processing", client.id);
    }

    if let Some((_, channel)) = client.channel() {
        if let Some(user) = client.user() {
            let _ = channel
                .send::<bool>(ChannelCommand::Disconnect { user_id: user.id })
                .await;
        }
    }

    if client.session_entered
        && let Some(user) = client.user()
    {
        User::delete_session(user.id).await;
    }
}

async fn process_guard(
    client: &mut Client,
    mut receiver: tokio::sync::mpsc::UnboundedReceiver<(EventId, Arc<dyn IEventData>)>,
) {
    'read_loop: loop {
        println!("Client {} waiting for data...", client.id);

        tokio::select! {
            result = receiver.recv() => {
                match result {
                    Some((EventId::Disconnect, _)) => {
                        break 'read_loop;
                    },
                    Some((id, event)) => events::handle_event(client, id, event).await,
                    _ => {
                        println!("Sender dropped, shutting down client");
                        break 'read_loop;
                    }
                }
            }

            result = client.read() => {
                match result {
                    Ok(0) => {
                        println!("Client disconnected");
                        break 'read_loop;
                    },
                    Ok(_) => {
                        const TLS_CLIENT_HELLO: &[u8] = b"\x16\x03\x01";

                        if is_http_request(&client.data) {
                            const HTTP_RICKROLL_REDIRECT: &[u8] = b"HTTP/1.1 301 Moved Permanently\r\nLocation: https://www.youtube.com/watch?v=dQw4w9WgXcQ\r\n\r\n";

                            if let Err(e) = client.send(HTTP_RICKROLL_REDIRECT).await {
                                println!("[Error] Failed to send HTTP response: {}", e);
                            }

                            break 'read_loop;
                        }

                        if client.data.starts_with(TLS_CLIENT_HELLO) {
                            // Were not supporting TLS, so just close the connection
                            break 'read_loop;
                        }

                        routes::handle_request(client).await;
                    },
                    Err(e) => {
                        println!("[Error] Failed to read from client: {}", e);
                        break 'read_loop;
                    },
                }
            },
        }
    }
}

pub async fn run(
    token: tokio_util::sync::CancellationToken,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let channels = (0..10)
        .map(|i| {
            let path = crate::config::get_str("CHANNELS", &format!("CH{}", i + 1), "");
            if !path.is_empty() {
                let split_path = path.split(',').collect::<Vec<_>>();

                let path = split_path.get(0).unwrap_or(&"");
                let _max_user = split_path
                    .get(1)
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(120);

                let path = format!("./resources/data/{}", path);

                Some(crate::channel::make_channel(
                    token.clone(),
                    0,
                    i,
                    1000,
                    path,
                ))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let channels = futures::future::join_all(channels.into_iter().filter_map(|x| x))
        .await
        .into_iter()
        .filter_map(|x| x.ok())
        .collect::<Vec<_>>();

    setup_channels(channels);

    let port = crate::config::get::<u32>("GATEWAY", "GamePort", 16010);

    println!("Starting gateway server on 0.0.0.0:{}", port);

    let server = Server::<Client>::new(tcpserver::AddressType::Any, port as u16).await?;

    // Heartbeat task to send heartbeat messages to all channels every 5 seconds
    let task = crate::util::spawn_named("Heartbeat Task", {
        let token = token.clone();
        async move {
            loop {
                if token.is_cancelled() {
                    break;
                }

                for channel in GET_CHANNELS().iter() {
                    channel
                        .send::<()>(ChannelCommand::Heartbeat)
                        .await
                        .expect("Failed to send heartbeat");
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    });

    futures::future::select(
        server.run(token.clone(), tcpserver::pin!(process)).boxed(),
        task.boxed(),
    )
    .await;

    println!("Gateway server is shutting down...");

    Ok(())
}
