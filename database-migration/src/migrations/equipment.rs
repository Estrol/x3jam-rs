use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Equipment::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Equipment::Id)
                            .unsigned()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Equipment::UserId)
                            .big_unsigned()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Equipment::Instrument).unsigned().default(0))
                    .col(ColumnDef::new(Equipment::Hair).unsigned().default(0))
                    .col(ColumnDef::new(Equipment::Accessory).unsigned().default(0))
                    .col(ColumnDef::new(Equipment::Glove).unsigned().default(0))
                    .col(ColumnDef::new(Equipment::Necklace).unsigned().default(0))
                    .col(ColumnDef::new(Equipment::Cloth).unsigned().default(0))
                    .col(ColumnDef::new(Equipment::Pant).unsigned().default(0))
                    .col(ColumnDef::new(Equipment::Glass).unsigned().default(0))
                    .col(ColumnDef::new(Equipment::Earring).unsigned().default(0))
                    .col(
                        ColumnDef::new(Equipment::ClothAccessory)
                            .unsigned()
                            .default(0),
                    )
                    .col(ColumnDef::new(Equipment::Shoes).unsigned().default(0))
                    .col(ColumnDef::new(Equipment::Face).unsigned().default(0))
                    .col(ColumnDef::new(Equipment::Wing).unsigned().default(0))
                    .col(
                        ColumnDef::new(Equipment::HairAccessory)
                            .unsigned()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Equipment::InstrumentAccessory)
                            .unsigned()
                            .default(0),
                    )
                    .col(ColumnDef::new(Equipment::Pet).unsigned().default(0))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_equipment_user_id")
                            .from(Equipment::Table, Equipment::UserId)
                            .to(super::user::User::Table, super::user::User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Equipment::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Equipment {
    #[iden = "equipment"]
    Table,
    #[iden = "id"]
    Id,
    #[iden = "user_id"]
    UserId,
    #[iden = "instrument"]
    Instrument,
    #[iden = "hair"]
    Hair,
    #[iden = "accessory"]
    Accessory,
    #[iden = "glove"]
    Glove,
    #[iden = "necklace"]
    Necklace,
    #[iden = "cloth"]
    Cloth,
    #[iden = "pant"]
    Pant,
    #[iden = "glass"]
    Glass,
    #[iden = "earring"]
    Earring,
    #[iden = "cloth_accessory"]
    ClothAccessory,
    #[iden = "shoes"]
    Shoes,
    #[iden = "face"]
    Face,
    #[iden = "wing"]
    Wing,
    #[iden = "hair_accessory"]
    HairAccessory,
    #[iden = "instrument_accessory"]
    InstrumentAccessory,
    #[iden = "pet"]
    Pet,
}
