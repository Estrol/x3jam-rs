#[gateway_derive::route(RequestId::ShopEnter)]
async fn shop_enter(_client: &mut super::Client, _packet: &mut super::Packet) {}

#[gateway_derive::route(RequestId::ShopLeave)]
async fn shop_leave(_client: &mut super::Client, _packet: &mut super::Packet) {}
