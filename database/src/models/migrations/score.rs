use sea_orm_migration::prelude::*;
use sea_query::Iden;

// Local identifier for the users table and its columns
#[derive(Iden)]
pub enum Score {
    #[iden = "scores"]
    Table,
    #[iden = "id"]
    Id,
    #[iden = "user_id"]
    UserId,
    #[iden = "music_id"]
    MusicId,
    #[iden = "score"]
    Score,
    #[iden = "cool"]
    Cool,
    #[iden = "good"]
    Good,
    #[iden = "bad"]
    Bad,
    #[iden = "miss"]
    Miss,
    #[iden = "max_combo"]
    MaxCombo,
    #[iden = "jam_combo"]
    JamCombo,
    #[iden = "timing"]
    Timing,
    #[iden = "rate"]
    Rate,
    #[iden = "fln"]
    Fln,
    #[iden = "sln"]
    Sln,
    #[iden = "nln"]
    Nln,
    #[iden = "arragement"]
    Arragement,
    #[iden = "skills"]
    Skills,
    #[iden = "timestamp"]
    Timestamp,
}

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260613_create_score"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Score::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Score::Id)
                            .unsigned()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Score::UserId)
                            .big_unsigned()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Score::MusicId)
                            .unsigned()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Score::Score).unsigned().not_null())
                    .col(ColumnDef::new(Score::Cool).unsigned().not_null())
                    .col(ColumnDef::new(Score::Good).unsigned().not_null())
                    .col(ColumnDef::new(Score::Bad).unsigned().not_null())
                    .col(ColumnDef::new(Score::Miss).unsigned().not_null())
                    .col(ColumnDef::new(Score::MaxCombo).unsigned().not_null())
                    .col(ColumnDef::new(Score::JamCombo).unsigned().not_null())
                    .col(ColumnDef::new(Score::Timing).unsigned().not_null())
                    .col(ColumnDef::new(Score::Rate).float().not_null())
                    .col(ColumnDef::new(Score::Fln).unsigned().not_null())
                    .col(ColumnDef::new(Score::Sln).unsigned().not_null())
                    .col(ColumnDef::new(Score::Nln).unsigned().not_null())
                    .col(ColumnDef::new(Score::Arragement).string().not_null())
                    .col(ColumnDef::new(Score::Skills).string().not_null())
                    .col(ColumnDef::new(Score::Timestamp).date_time().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Score::Table).to_owned())
            .await
    }
}
