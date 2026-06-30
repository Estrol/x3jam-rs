use sea_orm_migration::prelude::*;
use sea_query::Iden;

// Local identifier for the users table and its columns
#[derive(Iden)]
pub enum User {
    #[iden = "users"]
    Table,
    #[iden = "id"]
    Id,
    #[iden = "name"]
    Name,
    #[iden = "password_hash"]
    PasswordHash,
    #[iden = "nickname"]
    Nickname,
    #[iden = "exp"]
    Exp,
    #[iden = "wins"]
    Wins,
    #[iden = "losses"]
    Losses,
    #[iden = "draws"]
    Draws,
    #[iden = "mcash"]
    Mcash,
    #[iden = "point"]
    Point,
    #[iden = "o2gems"]
    O2gems,
    #[iden = "gender"]
    Gender,
}

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260613_create_user"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(User::Id)
                            .big_unsigned()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(User::Name).string().not_null())
                    .col(ColumnDef::new(User::PasswordHash).string().not_null())
                    .col(ColumnDef::new(User::Nickname).string().not_null())
                    .col(
                        ColumnDef::new(User::Exp)
                            .big_unsigned()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(User::Wins).unsigned().not_null().default(0))
                    .col(
                        ColumnDef::new(User::Losses)
                            .unsigned()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(User::Draws).unsigned().not_null().default(0))
                    .col(ColumnDef::new(User::Mcash).unsigned().not_null().default(0))
                    .col(ColumnDef::new(User::Point).unsigned().not_null().default(0))
                    .col(
                        ColumnDef::new(User::O2gems)
                            .unsigned()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(User::Gender)
                            .unsigned()
                            .not_null()
                            .default(0),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await
    }
}
