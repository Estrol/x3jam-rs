use crate::{channel::ChannelCommand, gateway::commands::ResponseId, user::User};

#[derive(Debug, Clone, encoder::StructDeserializer)]
pub struct InventoryEquipRequest {
    pub character_slot: u32,
    pub item_slot: u32,
}

#[derive(Debug, Clone, encoder::StructSerializer, Default)]
pub struct InventoryEquipResponse {
    pub result: u32,
    pub character_slot: u32,
    pub new_equip_item_id: u32,
    pub inventory_item_id: u32,
    pub old_equip_item_id: u32,
}

#[gateway_derive::route(RequestId::EquipItem)]
pub async fn handle_inventory_equip(
    client: &mut crate::gateway::Client,
    packet: &InventoryEquipRequest,
) {
    let Some((user_id, room)) = client.channel() else {
        println!("Received InventoryEquip request but client is not in a channel");
        return;
    };

    let Ok((res, user)) = room
        .send::<(InventoryEquipResponse, User)>(ChannelCommand::EquipItem {
            user_id,
            character_slot: packet.character_slot,
            item_slot: packet.item_slot,
        })
        .await
    else {
        println!("Failed to send InventoryEquip command to channel");
        return;
    };

    client.user = Some(user);

    client
        .send_packet(ResponseId::EquipItem, &res)
        .await
        .expect("Failed to send InventoryEquip response to client");
}
