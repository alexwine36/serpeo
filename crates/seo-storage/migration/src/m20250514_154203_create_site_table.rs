use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .create_table(
                Table::create()
                    .table(Site::Table)
                    .if_not_exists()
                    .col(pk_auto(Site::Id))
                    .col(string(Site::Name).default(""))
                    .col(string(Site::Url).not_null().unique_key())
                    .col(
                        ColumnDef::new(Site::CreatedAt)
                            .timestamp()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .drop_table(Table::drop().table(Site::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Site {
    Table,
    Id,
    Name,
    Url,
    CreatedAt,
}
