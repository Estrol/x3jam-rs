use std::io;
use tokio::fs::File;
use tokio::io::{AsyncRead, AsyncReadExt as _, BufReader};

pub mod game_modifier;
pub mod item_gender;

pub use game_modifier::{GameModifier, GameModifierType};
pub use item_gender::ItemGender;

#[derive(Debug)]
pub struct Item {
    pub id: u32,
    pub category: u8,
    pub planet: u8,
    pub gender: ItemGender,
    pub is_new: bool,
    pub quantity: u16,
    pub modifier: GameModifier,
    pub modifier_type: GameModifierType,
    pub currency: u8,
    pub price_gem: u32,
    pub price_cash: u32,
    pub name: Vec<u8>,
    pub description: Vec<u8>,
    pub frames: Vec<ItemFrame>,
}

impl Item {
    pub async fn read_from<R: AsyncRead + Unpin>(reader: &mut R) -> io::Result<Self> {
        let id = reader.read_u32_le().await?;
        let category = reader.read_u8().await?;
        let planet = reader.read_u8().await?;

        let flags = reader.read_u16_le().await?;
        let gender = ItemGender::from_flags(flags);
        let is_new = (flags >> 11) == 1;

        let quantity = reader.read_u16_le().await?;

        let modifier = GameModifier::from_u8(reader.read_u8().await?);
        let modifier_type = GameModifierType::from_u8(reader.read_u8().await?);

        let currency = reader.read_u8().await?;
        let price_gem = reader.read_u32_le().await?;
        let price_cash = reader.read_u32_le().await?;
        let _render_category = reader.read_u8().await?;

        let name = read_string(reader).await?;
        let description = read_string(reader).await?;

        let mut frames = Vec::new();
        const ITEM_COUNT: usize = 42;

        for _ in 0..ITEM_COUNT {
            let Some(reference) = read_optional_string(reader).await? else {
                continue;
            };

            frames.push(ItemFrame { reference });
        }

        Ok(Item {
            id,
            category,
            planet,
            gender,
            is_new,
            quantity,
            modifier,
            modifier_type,
            currency,
            price_gem,
            price_cash,
            name,
            description,
            frames,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct ItemFrame {
    pub reference: Vec<u8>,
}

#[derive(Debug)]
pub struct ItemList {
    pub items: Vec<Item>,
}

impl ItemList {
    pub async fn load_from_file(path: &str) -> io::Result<Self> {
        let file = File::open(path).await?;
        let mut reader = BufReader::new(file);

        let count = reader.read_u32_le().await?;
        let mut items = Vec::with_capacity(count as usize);

        for _ in 0..count {
            items.push(Item::read_from(&mut reader).await?);
        }

        Ok(Self { items })
    }

    pub fn get_item(&self, id: u32) -> Option<&Item> {
        self.items.iter().find(|item| item.id == id)
    }
}

/// Reads a u32 length prefix followed by a UTF-8 string.
async fn read_string<R: AsyncRead + Unpin>(reader: &mut R) -> io::Result<Vec<u8>> {
    let len = reader.read_u32_le().await?;

    let mut buf = vec![0; len as usize];
    reader.read_exact(&mut buf).await?;

    Ok(buf)
}

/// Reads a u8 validity flag, and if valid, reads a string.
async fn read_optional_string<R: AsyncRead + Unpin>(reader: &mut R) -> io::Result<Option<Vec<u8>>> {
    let valid = reader.read_u8().await?;
    if valid == 0 {
        return Ok(None);
    }

    Ok(Some(read_string(reader).await?))
}
