use sea_orm_migration::prelude::*;

use super::user::User;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "base_create_items"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Items::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Items::Id)
                            .unsigned()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Items::UserId).big_unsigned().not_null())
                    .col(ColumnDef::new(Items::Slot).unsigned().not_null())
                    .col(ColumnDef::new(Items::ItemId).unsigned().not_null())
                    .col(ColumnDef::new(Items::Quantity).unsigned().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_items_user_id")
                            .from(Items::Table, Items::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Items::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Items {
    #[iden = "items"]
    Table,
    #[iden = "id"]
    Id,
    #[iden = "user_id"]
    UserId,
    #[iden = "slot"]
    Slot,
    #[iden = "item_id"]
    ItemId,
    #[iden = "quantity"]
    Quantity,
}
