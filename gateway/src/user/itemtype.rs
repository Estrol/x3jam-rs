#[derive(Debug, Clone, Copy, encoder::StructSerializer, Default)]
pub enum ItemType {
    #[default]
    Instrument = 0,
    Hair = 1,
    Accessory = 2,
    Glove = 3,
    Necklace = 4,
    Cloth = 5,
    Pant = 6,
    Glasses = 7,
    Earring = 8,
    ClothAccessory = 9,
    Shoes = 10,
    Face = 11,
    Wing = 12,
    InstrumentAccessory = 13,
    Pet = 14,
    HairAccessory = 15,
}
