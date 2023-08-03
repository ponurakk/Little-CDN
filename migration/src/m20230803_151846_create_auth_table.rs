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
                    .table(Auth::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Auth::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Auth::UserId).integer().not_null())
                    .col(ColumnDef::new(Auth::Token).string().not_null())
                    .col(ColumnDef::new(Auth::ExpiresIn).timestamp().not_null())
                    .col(ColumnDef::new(Auth::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(Auth::UpdatedAt).timestamp().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-user-id")
                            .from(Auth::Table, Auth::UserId)
                            .to(Users::Table, Users::Id)
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Auth::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Auth {
    Table,
    Id,
    UserId,
    Token,
    ExpiresIn,
    CreatedAt,
    UpdatedAt,
}
