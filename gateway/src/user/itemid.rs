#[derive(Debug, Clone, Copy, encoder::StructSerializer, Default, encoder::StructDeserializer)]
pub struct ItemId {
    pub id: u32,
    pub amount: u32,
}

impl ItemId {
    pub fn new(id: u32) -> Self {
        Self { id, amount: 1 }
    }
}
