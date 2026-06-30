#[derive(Debug, Clone, Copy, encoder::StructSerializer, Default)]
pub struct ItemId {
    pub id: u32,
    pub amount: u32,
}
