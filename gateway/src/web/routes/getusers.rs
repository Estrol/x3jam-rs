pub async fn handle() -> impl axum::response::IntoResponse {
    let channels = crate::gateway::GET_CHANNELS().await;

    let mut result_channels = Vec::new();
    for channel in channels.iter() {
        let lock = channel.0.lock().await;

        let mut channel = ChannelInfo {
            region: channel.1,
            channel: channel.2,
            users: vec![],
        };

        for client in lock.users.iter() {
            let user = UserInfo {
                id: client.user.id,
                level: client.user.level(),
                nickname: client.user.nickname().to_string(),
            };

            channel.users.push(user);
        }

        result_channels.push(channel);
    }

    axum::Json(result_channels)
}

#[derive(serde::Serialize)]
pub struct ChannelInfo {
    region: u32,
    channel: u32,
    users: Vec<UserInfo>,
}

#[derive(serde::Serialize)]
pub struct UserInfo {
    pub id: u64,
    pub level: u32,
    pub nickname: String,
}