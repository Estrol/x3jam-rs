use sea_orm_migration::prelude::*;
use sea_query::Iden;

// Local identifier for the users table and its columns
#[derive(Iden)]
pub enum User {
    #[iden = "users"]
    Table,
    #[iden = "admin"]
    Admin,
}

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "782026_create_user"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(User::Table)
                    .add_column(
                        ColumnDef::new(User::Admin)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(User::Table)
                    .drop_column(User::Admin)
                    .to_owned(),
            )
            .await
    }
}
