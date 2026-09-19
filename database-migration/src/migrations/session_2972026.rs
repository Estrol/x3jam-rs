use sea_orm_migration::prelude::*;
use sea_query::Iden;

#[derive(Iden)]
pub enum Session {
    #[iden = "sessions"]
    Table,
    #[iden = "token"]
    Token,
    #[iden = "expiration"]
    Expiration,
    #[iden = "channel_id"]
    Channel,
    #[iden = "region"]
    Region,
    #[iden = "socket_id"]
    Socket,
}

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "2972026_create_session"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Session::Table)
                    .add_column(
                        ColumnDef::new(Session::Token)
                            .string()
                            .not_null()
                    )
                    .add_column(
                        ColumnDef::new(Session::Expiration)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default("1970-01-01 00:00:01"),
                    )
                    .add_column(
                        ColumnDef::new(Session::Channel)
                            .unsigned()
                            .null()
                    )
                    .add_column(
                        ColumnDef::new(Session::Region)
                            .unsigned()
                            .null()
                    )
                    .add_column(
                        ColumnDef::new(Session::Socket)
                            .unsigned()
                            .null()
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Session::Table)
                    .drop_column(Session::Token)
                    .drop_column(Session::Expiration)
                    .drop_column(Session::Channel)
                    .drop_column(Session::Region)
                    .drop_column(Session::Socket)
                    .to_owned(),
            )
            .await
    }
}
