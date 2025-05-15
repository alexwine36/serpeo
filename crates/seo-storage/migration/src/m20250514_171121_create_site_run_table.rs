use sea_orm_migration::{prelude::*, schema::*};

use crate::m20250514_154203_create_site_table::Site;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SiteRun::Table)
                    .if_not_exists()
                    .col(pk_auto(SiteRun::Id))
                    .col(integer(SiteRun::SiteId))
                    .col(
                        ColumnDef::new(SiteRun::CreatedAt)
                            .timestamp()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .col(
                        // TODO: Figure out how to define Sqlite enum
                        string(SiteRun::Status),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_site_run_site_id")
                            .from(SiteRun::Table, SiteRun::SiteId)
                            .to(Site::Table, Site::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SiteRun::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum SiteRun {
    Table,
    Id,
    SiteId,
    CreatedAt,
    Status,
}
