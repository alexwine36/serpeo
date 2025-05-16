use sea_orm_migration::{prelude::*, schema::*};

use crate::m20250514_211317_create_site_page_table::SitePage;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PageRuleResult::Table)
                    .if_not_exists()
                    .col(pk_auto(PageRuleResult::Id))
                    .col(integer(PageRuleResult::SitePageId))
                    .col(string(PageRuleResult::RuleId))
                    .col(boolean(PageRuleResult::Passed))
                    .col(
                        ColumnDef::new(PageRuleResult::CreatedAt)
                            .timestamp()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .index(
                        Index::create()
                            .name("idx_uniq_site_page_id_and_rule_id")
                            .col(PageRuleResult::SitePageId)
                            .col(PageRuleResult::RuleId)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_page_rule_result_site_page_id")
                            .from(PageRuleResult::Table, PageRuleResult::SitePageId)
                            .to(SitePage::Table, SitePage::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PageRuleResult::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum PageRuleResult {
    Table,
    Id,
    SitePageId,
    RuleId,
    Passed,
    CreatedAt,
}
