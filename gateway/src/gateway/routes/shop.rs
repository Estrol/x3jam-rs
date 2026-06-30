#[gateway_derive::route(RequestId::ShopLeave)]
async fn enter_shop(_client: &mut super::Client, _packet: &()) {
    // The game sends this packet when the player opens the shop, but it doesn't seem to expect a response for it, so we just ignore it.
}

#[gateway_derive::route(RequestId::ShopEnter)]
async fn leave_shop(_client: &mut super::Client, _packet: &()) {
    // The game sends this packet when the player closes the shop, but it doesn't seem to expect a response for it, so we just ignore it.
}
