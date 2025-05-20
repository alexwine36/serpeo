use sea_orm_migration::{prelude::*, schema::*};

use crate::{
    m20250514_154203_create_site_table::Site, m20250514_171121_create_site_run_table::SiteRun,
    m20250514_211317_create_site_page_table::SitePage,
    m20250516_171758_create_plugin_rule_table::PluginRule,
};

#[derive(DeriveMigrationName)]
pub struct Migration;
const FK_PAGE_RULE_RESULT_PLUGIN_RULE: &str = "fk_page_rule_result_plugin_rule";
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PageRuleResult::Table)
                    .if_not_exists()
                    .col(pk_auto(PageRuleResult::Id))
                    .col(integer(PageRuleResult::SiteId))
                    .col(integer(PageRuleResult::SiteRunId))
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
                            .name(FK_PAGE_RULE_RESULT_PLUGIN_RULE)
                            .from(PageRuleResult::Table, PageRuleResult::RuleId)
                            .to(PluginRule::Table, PluginRule::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_page_rule_result_site_page_id")
                            .from(PageRuleResult::Table, PageRuleResult::SitePageId)
                            .to(SitePage::Table, SitePage::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_page_rule_result_site_run_id")
                            .from(PageRuleResult::Table, PageRuleResult::SiteRunId)
                            .to(SiteRun::Table, SiteRun::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_page_rule_result_site_id")
                            .from(PageRuleResult::Table, PageRuleResult::SiteId)
                            .to(Site::Table, Site::Id),
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
    SiteId,
    SiteRunId,
    SitePageId,
    RuleId,
    Passed,
    CreatedAt,
}
