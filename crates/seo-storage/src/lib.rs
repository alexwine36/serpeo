use entities::prelude::{PageRuleResult, PluginRule, SitePage, SiteRun};
use entities::{page_rule_result, plugin_rule, site, site_page, site_run};
use enums::db_link_type::DbLinkType;
use enums::site_run_status::SiteRunStatus;
use migration::{Migrator, MigratorTrait, OnConflict};
use sea_orm::ConnectOptions;
use sea_orm::*;
use sea_orm::{Database, DbErr};
use seo_plugins::site_analyzer::{CrawlResult, PageLink};
use seo_plugins::utils::config::{RuleResult, SiteCheckContext};
use seo_plugins::utils::link_parser::LinkType;
use seo_plugins::utils::registry::PluginRegistry;
use serde::{Deserialize, Serialize};
use utils::category_counts::{CategoryResultDisplay, CategoryResultHistory};
use utils::category_detail::CategoryDetailResponse;
pub mod entities;
pub mod enums;
pub mod utils;
use crate::entities::prelude::Site;

const DATABASE_URL: &str = "sqlite::memory:";

#[derive(Clone)]
pub struct SeoStorage {
    db: DatabaseConnection,
}

impl SeoStorage {
    // Utilities

    /* #region Utilities */
    pub async fn new(db_url: &str) -> Self {
        let mut options = ConnectOptions::from(db_url.to_string());
        options.max_connections(5);
        options.sqlx_logging(true);
        let db = Database::connect(options).await.unwrap();
        SeoStorage { db }
    }

    pub async fn new_with_default() -> Self {
        let seo_storage = SeoStorage::new(DATABASE_URL).await;
        let _ = seo_storage.migrate_up().await;
        seo_storage
    }

    pub async fn new_migrated_with_default() -> Self {
        let seo_storage = SeoStorage::new_with_default().await;
        let _ = seo_storage.migrate_up().await;
        seo_storage
    }

    pub fn get_db(&self) -> DatabaseConnection {
        self.db.clone()
    }

    pub async fn migrate_up(&self) -> Result<(), DbErr> {
        if let Err(e) = Migrator::up(&self.db, None).await {
            println!("migration error: {:?}", e);
            // TODO: remove this once we have a better way to handle migration failures
            Migrator::fresh(&self.db).await?;
            Migrator::up(&self.db, None).await?;
        }

        self.seed_plugin_rule_table().await?;
        Ok(())
    }
    pub async fn migrate_reset(&self) -> Result<(), DbErr> {
        Migrator::reset(&self.db).await.unwrap();
        Ok(())
    }

    pub async fn seed_plugin_rule_table(&self) -> Result<(), DbErr> {
        let registry = PluginRegistry::default_with_config();
        let rules = registry.get_available_rules();

        for rule in rules {
            let rule = plugin_rule::ActiveModel {
                id: ActiveValue::Set(rule.id),
                name: ActiveValue::Set(rule.name),
                description: ActiveValue::Set(rule.description),
                category: ActiveValue::Set(rule.category.into()),
                severity: ActiveValue::Set(rule.severity.into()),
                plugin_name: ActiveValue::Set(rule.plugin_name),
                rule_type: ActiveValue::Set(rule.rule_type.into()),
                passed_message: ActiveValue::Set(rule.passed_message),
                failed_message: ActiveValue::Set(rule.failed_message),
                enabled: ActiveValue::Set(true),
                ..Default::default()
            };
            let on_conflict = OnConflict::column(plugin_rule::Column::Id)
                .update_columns([
                    plugin_rule::Column::Name,
                    plugin_rule::Column::Description,
                    plugin_rule::Column::Category,
                    plugin_rule::Column::Severity,
                    plugin_rule::Column::PluginName,
                    plugin_rule::Column::RuleType,
                    plugin_rule::Column::PassedMessage,
                    plugin_rule::Column::FailedMessage,
                    // plugin_rule::Column::Enabled,
                ])
                .to_owned();

            let _ = PluginRule::insert(rule)
                .on_conflict(on_conflict)
                .exec(&self.db)
                .await
                .unwrap();
        }
        Ok(())
    }
    /* #endregion */

    // Database interaction

    /* #region Site */
    pub async fn get_sites(
        &self,
    ) -> Result<Vec<utils::sites_with_site_runs::SiteWithSiteRuns>, DbErr> {
        let sites: Vec<(site::Model, Vec<site_run::Model>)> = Site::find()
            .find_with_related(site_run::Entity)
            .all(&self.db)
            .await
            .unwrap();
        Ok(utils::sites_with_site_runs::get_sites_with_site_runs(sites))
    }

    pub async fn get_site_by_id(&self, id: i32) -> Result<site::Model, DbErr> {
        let site = Site::find_by_id(id).one(&self.db).await?;
        Ok(site.unwrap())
    }

    pub async fn upsert_site(&self, url: &str) -> Result<i32, DbErr> {
        let site = Site::find()
            .filter(site::Column::Url.eq(url.to_string()))
            .one(&self.db)
            .await?;
        if site.is_some() {
            return Ok(site.unwrap().id);
        }

        let site = site::ActiveModel {
            url: ActiveValue::Set(url.to_string()),
            ..Default::default()
        };

        let on_conflict = OnConflict::column(site::Column::Url)
            .update_column(site::Column::Url)
            .to_owned();

        let res = Site::insert(site)
            .on_conflict(on_conflict)
            .exec(&self.db)
            .await
            .unwrap();

        Ok(res.last_insert_id)
    }
    /* #endregion */

    /* #region SiteRun */
    pub async fn get_site_runs(&self, site_id: i32) -> Result<Vec<site_run::Model>, DbErr> {
        let site_runs = SiteRun::find()
            .filter(site_run::Column::SiteId.eq(site_id))
            .all(&self.db)
            .await?;
        Ok(site_runs)
    }

    pub async fn create_site_run(&self, url: &str) -> Result<i32, DbErr> {
        let site = self.upsert_site(url).await?;
        let site_run = site_run::ActiveModel {
            site_id: ActiveValue::Set(site),
            status: ActiveValue::Set(SiteRunStatus::Pending),
            ..Default::default()
        };

        let res = SiteRun::insert(site_run).exec(&self.db).await.unwrap();

        Ok(res.last_insert_id)
    }
    pub async fn update_site_run_status(
        &self,
        id: i32,
        status: SiteRunStatus,
    ) -> Result<site_run::Model, DbErr> {
        let site_run = SiteRun::find_by_id(id).one(&self.db).await?;

        if site_run.is_some() {
            let site_run = site_run::ActiveModel {
                id: ActiveValue::Set(id),
                status: ActiveValue::Set(status),
                ..Default::default()
            };
            Ok(site_run.update(&self.db).await.unwrap())
        } else {
            Err(DbErr::RecordNotFound("SiteRun not found".to_string()))
        }
    }

    pub async fn get_site_run_by_id(&self, id: i32) -> Result<site_run::Model, DbErr> {
        let site_run = SiteRun::find_by_id(id).one(&self.db).await?;
        Ok(site_run.unwrap())
    }

    pub async fn get_site_run_link_counts(
        &self,
        site_run_id: i32,
    ) -> Result<Vec<SitePageLinkCount>, DbErr> {
        let site_pages = SitePage::find()
            .filter(site_page::Column::SiteRunId.eq(site_run_id))
            .select_only()
            .column(site_page::Column::DbLinkType)
            .column_as(site_page::Column::Url.count(), "count")
            .group_by(site_page::Column::DbLinkType)
            .into_model::<SitePageLinkCount>()
            .all(&self.db)
            .await?;

        Ok(site_pages)
    }

    /* #endregion */

    /* #region SitePage */

    pub async fn upsert_site_page(
        &self,
        site_run_id: i32,
        url: &str,
        link_type: LinkType,
    ) -> Result<site_page::Model, DbErr> {
        let site = self.get_site_run_by_id(site_run_id).await?;

        let site_page = SitePage::find()
            .filter(site_page::Column::Url.eq(url.to_string()))
            .filter(site_page::Column::SiteRunId.eq(site_run_id))
            .one(&self.db)
            .await?;

        if site_page.is_some() {
            return Ok(site_page.unwrap());
        }
        let site_page = site_page::ActiveModel {
            site_run_id: ActiveValue::Set(site_run_id),
            site_id: ActiveValue::Set(site.site_id),
            url: ActiveValue::Set(url.to_string()),
            db_link_type: ActiveValue::Set(DbLinkType::from(link_type)),
            ..Default::default()
        };

        let on_conflict =
            OnConflict::columns([site_page::Column::Url, site_page::Column::SiteRunId])
                .update_column(site_page::Column::Url)
                .to_owned();

        let res = SitePage::insert(site_page)
            .on_conflict(on_conflict)
            .exec(&self.db)
            .await
            .unwrap();

        let site_page = SitePage::find_by_id(res.last_insert_id)
            .one(&self.db)
            .await?;
        Ok(site_page.unwrap())
    }
    /* #endregion */

    /* #region PageRuleResult */

    pub async fn insert_many_page_rule_results(
        &self,
        site_run_id: i32,
        page_results: PageLink,
    ) -> Result<(), DbErr> {
        let url = page_results.url;
        let link_type = page_results.link_type;
        let site_page = self.upsert_site_page(site_run_id, &url, link_type).await?;

        let mut rule_results = vec![];

        let page_results = page_results.result;

        if let Some(page_results) = page_results {
            for rule_result in page_results.results {
                rule_results.push(self.format_rule_result(
                    site_page.id,
                    site_run_id,
                    site_page.site_id,
                    &rule_result.rule_id,
                    rule_result.passed,
                ));
            }

            let on_conflict = OnConflict::columns([
                page_rule_result::Column::SitePageId,
                page_rule_result::Column::RuleId,
            ])
            .update_column(page_rule_result::Column::Passed)
            .to_owned();

            if !rule_results.is_empty() {
                let res = PageRuleResult::insert_many(rule_results)
                    .on_conflict(on_conflict)
                    .exec(&self.db)
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn insert_many_site_rule_results(
        &self,
        site_run_id: i32,
        site_rule_results: Vec<RuleResult>,
    ) -> Result<(), DbErr> {
        for rule_result in site_rule_results {
            let rule_result_clone = rule_result.clone();
            match rule_result.context {
                SiteCheckContext::Urls(urls) => {
                    for url in urls {
                        let site_page = self
                            .upsert_site_page(site_run_id, &url, LinkType::Internal)
                            .await?;
                        let site_page_id = site_page.id;
                        let site_rule_result = self.format_rule_result(
                            site_page_id,
                            site_run_id,
                            site_page.site_id,
                            &rule_result.rule_id,
                            rule_result.passed,
                        );
                        let _ = PageRuleResult::insert(site_rule_result)
                            .exec(&self.db)
                            .await
                            .unwrap();
                    }
                }
                SiteCheckContext::Values(values) => {
                    for (_key, urls) in values {
                        for url in urls {
                            let site_page = self
                                .upsert_site_page(site_run_id, &url, LinkType::Internal)
                                .await?;
                            let site_page_id = site_page.id;
                            let site_rule_result = self.format_rule_result(
                                site_page_id,
                                site_run_id,
                                site_page.site_id,
                                &rule_result_clone.rule_id,
                                rule_result_clone.passed,
                            );

                            let on_conflict = OnConflict::columns([
                                page_rule_result::Column::SitePageId,
                                page_rule_result::Column::RuleId,
                            ])
                            .update_column(page_rule_result::Column::Passed)
                            .to_owned();

                            let _ = PageRuleResult::insert(site_rule_result)
                                .on_conflict(on_conflict)
                                .exec(&self.db)
                                .await?;
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub async fn handle_crawl_result(
        &self,
        site_run_id: i32,
        crawl_result: CrawlResult,
    ) -> Result<(), DbErr> {
        let site_rule_results = crawl_result.site_result;
        let page_results = crawl_result.page_results;
        self.insert_many_site_rule_results(site_run_id, site_rule_results)
            .await?;
        for page_result in page_results {
            self.insert_many_page_rule_results(site_run_id, page_result)
                .await?;
        }
        Ok(())
    }

    fn format_rule_result(
        &self,
        site_page_id: i32,
        site_run_id: i32,
        site_id: i32,
        rule_id: &str,
        passed: bool,
    ) -> page_rule_result::ActiveModel {
        page_rule_result::ActiveModel {
            site_page_id: ActiveValue::Set(site_page_id),
            site_run_id: ActiveValue::Set(site_run_id),
            site_id: ActiveValue::Set(site_id),
            rule_id: ActiveValue::Set(rule_id.to_string()),
            passed: ActiveValue::Set(passed),
            ..Default::default()
        }
    }

    /* #endregion */

    /* #region Results Display */
    pub async fn get_category_result(
        &self,
        site_run_id: &i32,
    ) -> Result<CategoryResultDisplay, DbErr> {
        let res: Vec<(plugin_rule::Model, Vec<page_rule_result::Model>)> = PluginRule::find()
            .find_with_related(page_rule_result::Entity)
            .filter(page_rule_result::Column::SiteRunId.eq(site_run_id.to_owned()))
            .all(&self.db)
            .await?;

        let category_counts = utils::category_counts::get_category_counts(res);

        let category_result_display =
            utils::category_counts::get_category_result_display(category_counts);

        Ok(category_result_display)
    }

    pub async fn get_site_category_history(
        &self,
        site_id: &i32,
    ) -> Result<Vec<CategoryResultHistory>, DbErr> {
        let site_runs = SiteRun::find()
            .filter(site_run::Column::SiteId.eq(site_id.to_owned()))
            .order_by_asc(site_run::Column::CreatedAt)
            .all(&self.db)
            .await?;
        let mut category_result_displays = vec![];
        // TODO: Update query to get all site run categories
        // The get_site_category_history function processes each site run sequentially, making a separate database query for each run. This could be inefficient for sites with many runs.
        for site_run in site_runs {
            let category_result_display = self.get_category_result(&site_run.id).await?;
            category_result_displays.push(CategoryResultHistory {
                created_at: site_run.created_at,
                data: category_result_display.data,
            });
        }
        Ok(category_result_displays)
    }

    pub async fn get_category_result_detail(
        &self,
        site_run_id: &i32,
        passed: Option<bool>,
    ) -> Result<CategoryDetailResponse, DbErr> {
        let mut res = PageRuleResult::find()
            .filter(page_rule_result::Column::SiteRunId.eq(site_run_id.to_owned()))
            .find_also_related(site_page::Entity)
            .find_also_related(plugin_rule::Entity);

        if let Some(passed) = passed {
            res = res.filter(page_rule_result::Column::Passed.eq(passed));
        }

        let res = res.all(&self.db).await?;

        let category_detail = utils::category_detail::get_category_detail(res).unwrap();

        Ok(category_detail)
    }
    /* #endregion */
}

#[derive(Debug, Clone, FromQueryResult, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct SitePageLinkCount {
    pub db_link_type: DbLinkType,
    pub count: i32,
}

#[cfg(test)]
mod tests {

    use std::collections::HashSet;

    use migration::SchemaManager;
    use seo_plugins::{
        site_analyzer::PageResult,
        utils::config::{RuleCategory, RuleResult, Severity, SiteCheckContext},
    };

    use crate::enums::plugin_rule_enums::DbRuleCategory;

    use super::*;

    #[tokio::test]
    async fn migrations_run() {
        let seo_storage = SeoStorage::new_with_default().await;
        let db = seo_storage.get_db();
        let schema_manager = SchemaManager::new(&db);

        let _ = seo_storage.migrate_up().await;
        assert!(schema_manager.has_table("site").await.unwrap());
    }

    #[tokio::test]
    async fn it_should_create_sites() {
        let seo_storage = SeoStorage::new_migrated_with_default().await;
        let db = seo_storage.get_db();

        let test_url = "https://forest-fitness-website-1dfad0.gitlab.io/";

        let res = seo_storage.upsert_site(test_url).await.unwrap();

        let res2 = seo_storage.upsert_site(test_url).await.unwrap();
        assert_eq!(
            res, res2,
            "duplicate urls should return the matching id {}:{}",
            res, res2
        );

        let sites = Site::find().all(&db).await.unwrap();
        assert_eq!(sites.len(), 1);
    }

    #[tokio::test]
    async fn it_should_create_site_runs() {
        let seo_storage = SeoStorage::new_migrated_with_default().await;
        let _ = seo_storage
            .upsert_site("https://other-site.com")
            .await
            .unwrap();
        let test_url = "https://forest-fitness-website-1dfad0.gitlab.io/";
        let cur_site = seo_storage.upsert_site(test_url).await.unwrap();
        let site_run_id = seo_storage.create_site_run(test_url).await.unwrap();
        assert_eq!(site_run_id, 1);

        let site_runs = SiteRun::find().all(&seo_storage.get_db()).await.unwrap();
        assert_eq!(site_runs.len(), 1);

        // It should be pending
        assert_eq!(site_runs[0].status, SiteRunStatus::Pending);
        assert_eq!(site_runs[0].site_id, cur_site);
    }

    #[tokio::test]
    async fn it_should_update_site_run_status() {
        let seo_storage = SeoStorage::new_migrated_with_default().await;
        let _ = seo_storage.migrate_up().await;

        let site_run_id = seo_storage
            .create_site_run("https://forest-fitness-website-1dfad0.gitlab.io/")
            .await
            .unwrap();
        let site_run = seo_storage
            .update_site_run_status(site_run_id, SiteRunStatus::Running)
            .await
            .unwrap();
        assert_eq!(site_run.status, SiteRunStatus::Running);

        let found_site_run = SiteRun::find_by_id(site_run_id)
            .one(&seo_storage.get_db())
            .await
            .unwrap();
        assert_eq!(found_site_run.unwrap().status, SiteRunStatus::Running);
    }

    #[tokio::test]
    async fn it_should_upsert_site_pages() {
        let seo_storage = SeoStorage::new_migrated_with_default().await;

        let site_run_id = seo_storage
            .create_site_run("https://forest-fitness-website-1dfad0.gitlab.io/")
            .await
            .unwrap();
        let site_page = seo_storage
            .upsert_site_page(
                site_run_id,
                "https://forest-fitness-website-1dfad0.gitlab.io/",
                LinkType::Internal,
            )
            .await
            .unwrap();
        let duplicate_site_page = seo_storage
            .upsert_site_page(
                site_run_id,
                "https://forest-fitness-website-1dfad0.gitlab.io/",
                LinkType::Internal,
            )
            .await
            .unwrap();
        assert_eq!(site_page.id, 1);
        assert_eq!(duplicate_site_page.id, 1);
        let site_pages = SitePage::find().all(&seo_storage.get_db()).await.unwrap();
        assert_eq!(site_pages.len(), 1);
    }

    #[tokio::test]
    async fn it_should_insert_many_page_rule_results() {
        let seo_storage = SeoStorage::new_migrated_with_default().await;
        let site_run_id = seo_storage
            .create_site_run("https://forest-fitness-website-1dfad0.gitlab.io/")
            .await
            .unwrap();

        let test_page_results = PageLink {
            url: "https://forest-fitness-website-1dfad0.gitlab.io/".to_string(),
            link_type: LinkType::Internal,
            found_in: HashSet::new(),
            result: Some(PageResult {
                error: false,
                results: vec![RuleResult {
                    rule_id: "title.has_title".to_string(),
                    name: "test".to_string(),
                    plugin_name: "test".to_string(),
                    passed: true,
                    message: "test".to_string(),
                    severity: Severity::Info,
                    category: RuleCategory::SEO,
                    context: SiteCheckContext::Empty,
                }],
            }),
        };
        let test_page_results_clone = test_page_results.clone();

        seo_storage
            .insert_many_page_rule_results(site_run_id, test_page_results)
            .await
            .unwrap();

        seo_storage
            .insert_many_page_rule_results(site_run_id, test_page_results_clone)
            .await
            .unwrap();

        let page_rule_results = PageRuleResult::find()
            .find_also_related(plugin_rule::Entity)
            .all(&seo_storage.get_db())
            .await
            .unwrap();
        assert_eq!(page_rule_results.len(), 1);
        println!("page_rule_results: {:?}", page_rule_results);

        let category_result = seo_storage.get_category_result(&site_run_id).await.unwrap();
        println!("category_result: {:?}", category_result);

        assert_eq!(category_result.data.len(), 1);
        assert_eq!(category_result.total, 1);
        assert_eq!(category_result.passed, 1);
        assert_eq!(category_result.failed, 0);
        assert_eq!(category_result.data[&DbRuleCategory::SEO].total, 1);
        assert_eq!(category_result.data[&DbRuleCategory::SEO].passed, 1);
        assert_eq!(category_result.data[&DbRuleCategory::SEO].failed, 0);

        let site_page_link_counts = seo_storage
            .get_site_run_link_counts(site_run_id)
            .await
            .unwrap();
        assert_eq!(site_page_link_counts.len(), 1);
        assert_eq!(site_page_link_counts[0].db_link_type, DbLinkType::Internal);
        assert_eq!(site_page_link_counts[0].count, 1);

        let test_page_results = PageLink {
            url: "https://forest-fitness-website-1dfad0.gitlab.io/".to_string(),
            link_type: LinkType::Internal,
            found_in: HashSet::new(),
            result: Some(PageResult {
                error: false,
                results: vec![RuleResult {
                    rule_id: "title.title_length".to_string(),
                    name: "test".to_string(),
                    plugin_name: "test".to_string(),
                    passed: false,
                    message: "test".to_string(),
                    severity: Severity::Info,
                    category: RuleCategory::SEO,
                    context: SiteCheckContext::Empty,
                }],
            }),
        };

        seo_storage
            .insert_many_page_rule_results(site_run_id, test_page_results)
            .await
            .unwrap();

        let category_detail = seo_storage
            .get_category_result_detail(&site_run_id, Some(false))
            .await
            .unwrap();
        println!("category_detail: {:?}", category_detail);

        assert_eq!(category_detail.data.len(), 1);
        assert_eq!(category_detail.data[&DbRuleCategory::SEO].len(), 1);
        let category_detail = seo_storage
            .get_category_result_detail(&site_run_id, None)
            .await
            .unwrap();
        println!("category_detail: {:?}", category_detail);
        assert_eq!(category_detail.data.len(), 1);
        assert_eq!(category_detail.data[&DbRuleCategory::SEO].len(), 2);
    }
}
