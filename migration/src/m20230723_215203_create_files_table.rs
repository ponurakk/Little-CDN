use sea_orm_migration::prelude::*;

use crate::m20230723_215117_create_users_table::Users;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Files::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Files::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Files::UserId).integer().not_null())
                    .col(ColumnDef::new(Files::Filename).string().not_null())
                    .col(ColumnDef::new(Files::Filetype).string().not_null())
                    .col(ColumnDef::new(Files::Size).integer().not_null())
                    .col(ColumnDef::new(Files::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(Files::UpdatedAt).timestamp().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-user-id")
                            .from(Files::Table, Files::UserId)
                            .to(Users::Table, Users::Id)
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Files::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Files {
    Table,
    Id,
    UserId,
    Filename,
    Filetype,
    Size,
    CreatedAt,
    UpdatedAt,
}
