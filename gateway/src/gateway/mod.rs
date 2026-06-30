pub mod cbc_decrypt;
pub mod commands;
pub mod events;
pub mod ojnlist;
pub mod routes;
pub mod stateful;

pub mod client;
pub mod itemlist;

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

static DATABASE: OnceLock<database::GameDatabase> = OnceLock::new();

pub async fn setup_database() {
    let db = database::GameDatabase::from_mysql("localhost:3306", "root", "", "otwotest").await;

    #[cfg(debug_assertions)]
    {
        if db.get_user_by_name("testuser").await.is_none() {
            let hash =
                bcrypt::hash("password", bcrypt::DEFAULT_COST).expect("Failed to hash password");

            db.create_user(
                "testuser",
                &hash,
                "TestUser",
                database::CharacterGender::Male,
            )
            .await
            .expect("Failed to create debug user");
        }

        if db.get_user_by_name("testuser2").await.is_none() {
            let hash =
                bcrypt::hash("password", bcrypt::DEFAULT_COST).expect("Failed to hash password");

            db.create_user(
                "testuser2",
                &hash,
                "TestUser2",
                database::CharacterGender::Female,
            )
            .await
            .expect("Failed to create debug user");
        }

        if db.get_user_by_name("testuser3").await.is_none() {
            let hash =
                bcrypt::hash("password", bcrypt::DEFAULT_COST).expect("Failed to hash password");

            db.create_user(
                "testuser3",
                &hash,
                "TestUser3",
                database::CharacterGender::Male,
            )
            .await
            .expect("Failed to create debug user");
        }

        if db.get_user_by_name("testuser4").await.is_none() {
            let hash =
                bcrypt::hash("password", bcrypt::DEFAULT_COST).expect("Failed to hash password");

            db.create_user(
                "testuser4",
                &hash,
                "TestUser4",
                database::CharacterGender::Male,
            )
            .await
            .expect("Failed to create debug user");
        }

        if db.get_user_by_name("testuser5").await.is_none() {
            let hash =
                bcrypt::hash("password", bcrypt::DEFAULT_COST).expect("Failed to hash password");

            db.create_user(
                "testuser5",
                &hash,
                "TestUser5",
                database::CharacterGender::Male,
            )
            .await
            .expect("Failed to create debug user");
        }

        if db.get_user_by_name("testuser6").await.is_none() {
            let hash =
                bcrypt::hash("password", bcrypt::DEFAULT_COST).expect("Failed to hash password");

            db.create_user(
                "testuser6",
                &hash,
                "TestUser6",
                database::CharacterGender::Male,
            )
            .await
            .expect("Failed to create debug user");
        }

        if db.get_user_by_name("testuser7").await.is_none() {
            let hash =
                bcrypt::hash("password", bcrypt::DEFAULT_COST).expect("Failed to hash password");

            db.create_user(
                "testuser7",
                &hash,
                "TestUser7",
                database::CharacterGender::Male,
            )
            .await
            .expect("Failed to create debug user");
        }

        if db.get_user_by_name("testuser8").await.is_none() {
            let hash =
                bcrypt::hash("password", bcrypt::DEFAULT_COST).expect("Failed to hash password");

            db.create_user(
                "testuser8",
                &hash,
                "TestUser8",
                database::CharacterGender::Male,
            )
            .await
            .expect("Failed to create debug user");
        }
    }

    DATABASE.set(db).expect("Failed to set database");
}

#[allow(non_snake_case)]
pub async fn GET_DATABASE() -> &'static database::GameDatabase {
    DATABASE.get().expect("Database not initialized")
}

lazy_static::lazy_static! {
    static ref ITEM_LIST: OnceLock<itemlist::ItemList> = OnceLock::new();
}

pub async fn setup_item_list() {
    let list = itemlist::ItemList::load_from_file("itemlist.dat")
        .await
        .expect("Failed to load item list");

    ITEM_LIST.set(list).expect("Failed to set item list");
}

#[allow(non_snake_case)]
pub fn GET_ITEM_LIST() -> &'static itemlist::ItemList {
    ITEM_LIST.get().expect("Item list not initialized")
}

lazy_static::lazy_static! {
    static ref CHANNELS: OnceLock<Vec<ChannelHandle>> = OnceLock::new();
}

#[allow(non_snake_case)]
pub fn GET_CHANNELS() -> &'static Vec<ChannelHandle> {
    CHANNELS.get().expect("Channels not initialized")
}

pub fn setup_channels(channels: Vec<ChannelHandle>) {
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
    channels: Vec<ChannelHandle>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Starting gateway server on port 16010...");

    setup_database().await;
    setup_item_list().await;
    setup_channels(channels);

    let server = Server::<Client>::new(tcpserver::AddressType::Any, 16010).await?;
    server.run(token, tcpserver::pin!(process)).await?;

    println!("Gateway server has shut down.");

    Ok(())
}
