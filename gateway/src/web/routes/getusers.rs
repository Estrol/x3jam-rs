pub fn calculate_level(exp: u64) -> u32 {
    let base_exp = 1000.0;
    let growth_factor = 1.5;

    if exp < base_exp as u64 {
        0
    } else {
        let level = ((exp as f64 / base_exp).log(growth_factor) + 1.0).floor();
        level as u32
    }
}

pub async fn get() -> impl axum::response::IntoResponse {
    let database = crate::database::get();
    let users = database
        .query_user_channels_list()
        .await
        .unwrap_or_default();

    let mut results: Vec<ChannelInfo> = Vec::new();
    for user in users {
        let user_info = UserInfo {
            level: calculate_level(user.exp),
            nickname: user.username,
        };

        match results
            .iter_mut()
            .find(|c| c.region == user.region && c.channel == user.channel_id)
        {
            Some(channel) => {
                channel.users.push(user_info);
            }
            None => {
                results.push(ChannelInfo {
                    region: user.region,
                    channel: user.channel_id,
                    users: vec![user_info],
                });
            }
        }
    }

    axum::Json(results)
}

#[derive(serde::Serialize)]
pub struct ChannelInfo {
    region: u32,
    channel: u32,
    users: Vec<UserInfo>,
}

#[derive(serde::Serialize)]
pub struct UserInfo {
    pub level: u32,
    pub nickname: String,
}
