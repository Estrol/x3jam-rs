use database::Equipment;

use crate::{
    channel::ChannelCommand,
    gateway::{commands::ResponseId, routes::listroom::EffectEntry},
    itemlist::GameModifierType,
    user::{ItemId, User},
};

#[gateway_derive::route(RequestId::ShopActionBuy)]
pub async fn shop_buy(client: &mut super::Client, _packet: &()) {
    if client.user().is_none() {
        println!("Received ShopActionBuy request but client has no user");
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
    pub inventory: [ItemId; 30],
    pub equipment: Equipment,
    pub music_cash: u32,
    pub item_cash: u32,
    pub effects: Vec<EffectEntry>,
}

#[gateway_derive::route(RequestId::ShopActionSync)]
pub async fn shop_sync(client: &mut super::Client, _packet: &()) {
    let Some((user_id, channel)) = client.channel() else {
        println!(
            "Client {} is not in a channel, cannot sync shop info",
            client.id
        );
        return;
    };

    let Ok(user) = channel
        .send::<User>(ChannelCommand::SyncUserInfo { user_id })
        .await
    else {
        println!(
            "Failed to sync user info for client {}: user not found",
            client.id
        );
        return;
    };

    let lists = crate::itemlist::get();

    let mut effects = Vec::new();
    for i in user.inventory.iter().filter(|item| item.id != 0) {
        let Some(item) = lists.get_item(i.id) else {
            continue;
        };

        if item.modifier_type != GameModifierType::None {
            effects.push(EffectEntry {
                id: i.id,
                amount: i.amount,
            });
        }
    }

    let response = ShopActionSyncResponse {
        gems: user.info.o2gems,
        mcash: user.info.mcash,
        o2cash: user.info.point,
        inventory: user.inventory(),
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
