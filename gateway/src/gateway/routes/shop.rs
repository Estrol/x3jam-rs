use database::Equipment;

use crate::{
    channel::ChannelCommand,
    gateway::{commands::ResponseId, routes::listroom::EffectEntry},
    itemlist::GameModifierType,
    user::User,
};

#[gateway_derive::route(RequestId::ShopActionBuy)]
pub async fn shop_buy(client: &mut super::Client, _packet: &()) {
    if client.user().is_none() {
        log::info!("Received ShopActionBuy request but client has no user");
        return;
    }

    client
        .send_packet(ResponseId::ShopActionBuy, &())
        .await
        .expect("Failed to send ShopActionBuy response to client");
}

#[derive(Debug, Clone, encoder::StructSerializer)]
pub struct ShopActionSyncResponse {
    pub gems: u32,
    pub mcash: u32,
    pub o2cash: u32,
    pub inventory: [u32; 30],
    pub equipment: Equipment,
    pub music_cash: u32,
    pub item_cash: u32,
    pub effects: Vec<EffectEntry>,
}

#[gateway_derive::route(RequestId::ShopActionSync)]
pub async fn shop_sync(client: &mut super::Client, _packet: &()) {
    let Some((user_id, channel)) = client.channel() else {
        log::info!(
            "Client {} is not in a channel, cannot sync shop info",
            client.id
        );
        return;
    };

    let Ok(user) = channel
        .send::<User>(ChannelCommand::SyncUserInfo { user_id })
        .await
    else {
        log::info!(
            "Failed to sync user info for client {}: user not found",
            client.id
        );
        return;
    };

    let lists = crate::itemlist::get();

    let mut effects = Vec::new();
    let mut inventory = [0u32; 30];

    for (i, item) in user.inventory.iter().enumerate() {
        let Some(item_info) = lists.get_item(item.id) else {
            continue;
        };

        if item_info.modifier_type != GameModifierType::None {
            effects.push(EffectEntry {
                id: item.id,
                amount: item.amount,
            });
        }

        inventory[i] = item.id;
    }

    let response = ShopActionSyncResponse {
        gems: user.info.o2gems,
        mcash: user.info.mcash,
        o2cash: user.info.point,
        inventory,
        equipment: user.equipment(),
        music_cash: 0,
        item_cash: 0,
        effects,
    };

    client.user = Some(user);

    client
        .send_packet(ResponseId::ShopActionSync, &response)
        .await
        .expect("Failed to send ShopActionSync response to client");
}
