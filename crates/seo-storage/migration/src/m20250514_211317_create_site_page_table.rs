use sea_orm_migration::{prelude::*, schema::*};

use crate::m20250514_171121_create_site_run_table::SiteRun;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SitePage::Table)
                    .if_not_exists()
                    .col(pk_auto(SitePage::Id))
                    .col(integer(SitePage::SiteRunId))
                    .col(string(SitePage::Url))
                    .col(
                        // TODO: Figure out how to define Sqlite enum
                        string(SitePage::DbLinkType),
                    )
                    .col(
                        ColumnDef::new(SitePage::CreatedAt)
                            .timestamp()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .index(
                        Index::create()
                            .name("idx_uniq_site_run_id_and_url")
                            .col(SitePage::SiteRunId)
                            .col(SitePage::Url)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_site_page_site_run_id")
                            .from(SitePage::Table, SitePage::SiteRunId)
                            .to(SiteRun::Table, SiteRun::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SitePage::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum SitePage {
    Table,
    Id,
    SiteRunId,
    Url,
    DbLinkType,
    CreatedAt,
}
