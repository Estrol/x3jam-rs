use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, Default, DeriveEntityModel)]
#[sea_orm(table_name = "scores")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: u32,
    #[sea_orm(unique)]
    pub user_id: u64,

    pub music_id: u32,
    pub score: u32,
    pub cool: u16,
    pub good: u16,
    pub bad: u16,
    pub miss: u16,
    pub max_combo: u16,
    pub jam_combo: u16,

    pub timing: u32,
    pub rate: f32,
    pub fln: u32,
    pub sln: u32,
    pub nln: u32,
    pub arragement: String, // Formatted as: "1234567" or "7654321"

    pub skills: String, // Formatted as: "skill1,skill2,skill3"
    pub timestamp: DateTimeUtc,
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
