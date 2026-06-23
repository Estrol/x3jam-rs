use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, DeriveEntityModel)]
#[sea_orm(table_name = "equipment")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: u32,
    #[sea_orm(unique_key)]
    pub user_id: u64,
    #[sea_orm(default = 0)]
    pub instrument: u32,
    #[sea_orm(default = 0)]
    pub hair: u32,
    #[sea_orm(default = 0)]
    pub accessory: u32,
    #[sea_orm(default = 0)]
    pub glove: u32,
    #[sea_orm(default = 0)]
    pub necklace: u32,
    #[sea_orm(default = 0)]
    pub cloth: u32,
    #[sea_orm(default = 0)]
    pub pant: u32,
    #[sea_orm(default = 0)]
    pub glass: u32,
    #[sea_orm(default = 0)]
    pub earring: u32,
    #[sea_orm(default = 0)]
    pub cloth_accessory: u32,
    #[sea_orm(default = 0)]
    pub shoes: u32,
    #[sea_orm(default = 0)]
    pub face: u32,
    #[sea_orm(default = 0)]
    pub wing: u32,
    #[sea_orm(default = 0)]
    pub hair_accessory: u32,
    #[sea_orm(default = 0)]
    pub instrument_accessory: u32,
    #[sea_orm(default = 0)]
    pub pet: u32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
}

impl ActiveModelBehavior for ActiveModel {}
