use std::sync::Arc;

use crate::gateway::{
    commands::{EventId, ResponseId}, events::connection::DisconnectEventArgs, user::{User, UserError}
};

#[derive(encoder::StructDeserializer)]
struct LoginRequest {
    username: std::ffi::CString,
    password: std::ffi::CString,
}

#[repr(i32)]
#[derive(Debug, Copy, Clone, encoder::StructSerializer, PartialEq, Eq)]
pub enum LoginResult {
    Success = 0,
    AlreadyLoggedIn = -2,
    InvalidCredentials = -1,
    GenericError = -101,
}

#[gateway_derive::route(RequestId::GatewayLogin)]
pub async fn login_proc(client: &mut super::Client, packet: &mut super::Packet) {
    match super::parse_request::<LoginRequest>(&packet.body) {
        Ok(request) => {
            let username = request.username.to_string_lossy().to_string();
            let password = request.password.to_string_lossy().to_string();

            println!(
                "Client {} is attempting to log in with username '{}'",
                client.id, username
            );

            let result = match User::verify_credentials(&username, &password).await {
                Ok(_) => LoginResult::Success,
                Err(UserError::InvalidCredentials) => LoginResult::InvalidCredentials,
                Err(UserError::Error(err)) => {
                    println!("[Error] Failed to verify credentials: {}", err);
                    LoginResult::GenericError
                }
            };

            client
                .send_packet(ResponseId::GatewayAuth, &result)
                .await
                .expect("Failed to send login response");
        }
        Err(e) => {
            println!("[Error] Failed to parse login request: {}", e);
        }
    }
}

#[derive(encoder::StructSerializer)]
struct VersionResponse {
    user_id: u32,
}

#[gateway_derive::route(RequestId::GatewayReauth)]
pub async fn reauth_proc(client: &mut super::Client, packet: &mut super::Packet) {
    match super::parse_request::<LoginRequest>(&packet.body) {
        Ok(request) => {
            let username = request.username.to_string_lossy().to_string();
            let password = request.password.to_string_lossy().to_string();

            println!(
                "Client {} is attempting to re-authenticate with username '{}'",
                client.id, username
            );

            let result = match User::verify_credentials(&username, &password).await {
                Ok(user_id) => {
                    if !User::try_create_session(user_id).await.unwrap_or(false) {
                        LoginResult::AlreadyLoggedIn
                    } else {
                        if let Ok(mut user) = User::request_user(user_id, true).await {
                            let Some(sender) = client.sender.as_ref() else {
                                println!("Client sender is not available");
                                return;
                            };

                            user.set_sender(Arc::clone(sender));
                            client.user = Some(user);
                            client.session_entered = true;

                            LoginResult::Success
                        } else {
                            User::delete_session(user_id).await;

                            LoginResult::GenericError
                        }
                    }
                }
                Err(UserError::InvalidCredentials) => LoginResult::InvalidCredentials,
                Err(UserError::Error(err)) => {
                    println!("[Error] Failed to verify credentials: {}", err);
                    LoginResult::GenericError
                }
            };

            client
                .send_packet(ResponseId::GatewayReauth, &result)
                .await
                .expect("Failed to send re-auth response");

            // O2Hook2 extensions
            if result == LoginResult::Success {
                let response = VersionResponse {
                    user_id: client.user.as_ref().unwrap().id as u32,
                };

                client
                    .send_packet(ResponseId::RequestVersion, &response)
                    .await
                    .expect("Failed to send version response");

                let test = std::ffi::CString::new("O2Hook2").unwrap();

                client
                    .send_packet(0xABCE as u16, &test)
                    .await
                    .expect("Failed to send re-auth completion packet");
            }
        }
        Err(e) => {
            println!("[Error] Failed to parse re-auth request: {}", e);
        }
    }
}

pub const fn str2int(bytes: &[u8]) -> u32 {
    let mut result: u32 = 5381;
    let mut i = 0;

    while i < bytes.len() {
        result = result.wrapping_mul(33)
            ^ (bytes[i] as u32);
        i += 1;
    }

    result
}

#[gateway_derive::route(RequestId::RequestVersion)]
pub async fn request_version_proc(client: &mut super::Client, _packet: &mut super::Packet) {
    #[derive(Debug, encoder::StructDeserializer)]
    #[allow(dead_code)]
    struct VersionRequest {
        version: u32,
        user_id: u32,
    }

    match super::parse_request::<VersionRequest>(&_packet.body) {
        Ok(data) => {
            const EXPECTED_VERSION: &str = "1.6.1";

            if data.version != str2int(EXPECTED_VERSION.as_bytes()) {
                client.send_packet(ResponseId::RejectVersion, &())
                    .await
                    .expect("Failed to send version response");

                let Some(sender) = client.sender.as_ref() else {
                    println!("Client sender is not available");
                    return;
                };

                sender.send((EventId::Disconnect, Arc::new(DisconnectEventArgs)))
                    .expect("Failed to send disconnect event");
            }
        }
        Err(e) => {
            println!("[Error] Failed to parse version request: {}", e);
        }
    }
}

#[gateway_derive::route(RequestId::GatewayConnect)]
pub async fn connect_proc(client: &mut super::Client, _packet: &mut super::Packet) {
    #[derive(encoder::StructSerializer, Default)]
    struct ConnectResponse {
        value: u16,
        unk2: u8,
        random_bytes: CArray<u8, 0x20>,
        random_bytes2: CArray<u8, 0x10>,
    }

    let mut response = ConnectResponse::default();
    response.value = 1;

    // getrandom::fill(&mut *response.random_bytes).expect("Failed to generate random bytes");
    // getrandom::fill(&mut *response.random_bytes2).expect("Failed to generate random bytes");

    // client.login_xor_char = response.random_bytes[0];

    client
        .send_packet(ResponseId::GatewayConnect, &response)
        .await
        .expect("Failed to send connect response");
}

#[gateway_derive::route(RequestId::GatewayReconnect)]
pub async fn gateway_connect_proc(client: &mut super::Client, _packet: &mut super::Packet) {
    #[derive(encoder::StructSerializer, Default)]
    struct ConnectResponse {
        value: u16,
        unk2: u8,
        random_bytes: CArray<u8, 0x10>,
        random_bytes2: CArray<u8, 0x20>,
    }

    let mut response = ConnectResponse::default();
    // response.value = 0xFFF;
    response.value = 1;

    // getrandom::fill(&mut *response.random_bytes).expect("Failed to generate random bytes");
    // getrandom::fill(&mut *response.random_bytes2).expect("Failed to generate random bytes");

    // client.login_xor_char = response.random_bytes2[0];

    client
        .send_packet(ResponseId::GatewayReconnect, &response)
        .await
        .expect("Failed to send gateway connect response");
}

pub struct CArray<T, const N: usize>([T; N]);

impl<T: encoder::StructEncodeImpl, const N: usize> encoder::StructEncodeImpl for CArray<T, N> {
    fn impl_encode(&self, writer: &mut impl std::io::Write) -> std::io::Result<()> {
        for item in &self.0 {
            item.impl_encode(writer)?;
        }
        Ok(())
    }
}

impl<T, const N: usize> std::ops::Deref for CArray<T, N> {
    type Target = [T; N];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, const N: usize> std::ops::DerefMut for CArray<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Default + Copy, const N: usize> Default for CArray<T, N> {
    fn default() -> Self {
        Self([T::default(); N])
    }
}
