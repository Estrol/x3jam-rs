use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, Default, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: u64,
    pub name: String,
    pub password_hash: String,
    pub nickname: String,
    pub email: String,
    #[sea_orm(default = false)]
    pub admin: bool,
    #[sea_orm(default = 0)]
    pub exp: u64,
    #[sea_orm(default = 0)]
    pub wins: u32,
    #[sea_orm(default = 0)]
    pub losses: u32,
    #[sea_orm(default = 0)]
    pub draws: u32,
    #[sea_orm(default = 0)]
    pub mcash: u32,
    #[sea_orm(default = 0)]
    pub point: u32,
    #[sea_orm(default = 0)]
    pub o2gems: u32,
    #[sea_orm(default = Gender::Male)]
    pub gender: Gender,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum, Default)]
#[sea_orm(rs_type = "u8", db_type = "TinyUnsigned")]
pub enum Gender {
    #[sea_orm(num_value = 0)]
    #[default]
    Male,
    #[sea_orm(num_value = 1)]
    Female,
}
