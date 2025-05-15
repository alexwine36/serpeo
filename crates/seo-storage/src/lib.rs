use entities::prelude::{PageRuleResult, SitePage, SiteRun};
use entities::{page_rule_result, site, site_page, site_run};
use enums::db_link_type::DbLinkType;
use enums::site_run_status::SiteRunStatus;
use migration::{Migrator, MigratorTrait, OnConflict};
use sea_orm::ConnectOptions;
use sea_orm::*;
use sea_orm::{Database, DbErr};
use seo_plugins::site_analyzer::{CrawlResult, PageLink};
use seo_plugins::utils::link_parser::LinkType;
pub mod entities;
pub mod enums;
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
        options.max_connections(10);
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
        Migrator::up(&self.db, None).await.unwrap();

        Ok(())
    }
    /* #endregion */

    // Database interaction

    /* #region Site */
    pub async fn get_sites(&self) -> Result<Vec<site::Model>, DbErr> {
        let sites = Site::find().all(&self.db).await.unwrap();
        Ok(sites)
    }

    pub async fn upsert_site(&self, url: &str) -> Result<i32, DbErr> {
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

        println!("site upsert: {:?}", res);

        Ok(res.last_insert_id)
    }
    /* #endregion */

    /* #region SiteRun */
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
    /* #endregion */

    /* #region SitePage */
    pub async fn upsert_site_page(
        &self,
        site_run_id: i32,
        url: &str,
        link_type: LinkType,
    ) -> Result<i32, DbErr> {
        let site_page = site_page::ActiveModel {
            site_run_id: ActiveValue::Set(site_run_id),
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

        Ok(res.last_insert_id)
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
        let site_page_id = self.upsert_site_page(site_run_id, &url, link_type).await?;

        let mut rule_results = vec![];

        for rule_result in page_results.result.unwrap().results {
            rule_results.push(self.format_rule_result(
                site_page_id,
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

        let res = PageRuleResult::insert_many(rule_results)
            .on_conflict(on_conflict)
            .exec(&self.db)
            .await
            .unwrap();

        println!("page rule results: {:?}", res);

        Ok(())
    }

    fn format_rule_result(
        &self,
        site_page_id: i32,
        rule_id: &str,
        passed: bool,
    ) -> page_rule_result::ActiveModel {
        page_rule_result::ActiveModel {
            site_page_id: ActiveValue::Set(site_page_id),
            rule_id: ActiveValue::Set(rule_id.to_string()),
            passed: ActiveValue::Set(passed),
            ..Default::default()
        }
    }

    pub async fn upsert_page_rule_result(
        &self,
        site_page_id: i32,
        rule_id: &str,
        passed: bool,
    ) -> Result<i32, DbErr> {
        let page_rule_result = self.format_rule_result(site_page_id, rule_id, passed);

        let res = PageRuleResult::insert(page_rule_result)
            .exec(&self.db)
            .await
            .unwrap();

        Ok(res.last_insert_id)
    }
    /* #endregion */
}

#[cfg(test)]
mod tests {

    use std::collections::HashSet;

    use migration::SchemaManager;
    use seo_plugins::{
        site_analyzer::PageResult,
        utils::config::{RuleCategory, RuleResult, Severity, SiteCheckContext},
    };

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
        println!("sites: {:?}", sites);
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
        println!("site_runs: {:?}", site_runs);

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
        println!("site_run: {:?}", site_run);
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
        let site_page_id = seo_storage
            .upsert_site_page(
                site_run_id,
                "https://forest-fitness-website-1dfad0.gitlab.io/",
                LinkType::Internal,
            )
            .await
            .unwrap();
        let duplicate_site_page_id = seo_storage
            .upsert_site_page(
                site_run_id,
                "https://forest-fitness-website-1dfad0.gitlab.io/",
                LinkType::Internal,
            )
            .await
            .unwrap();
        assert_eq!(site_page_id, 1);
        assert_eq!(duplicate_site_page_id, 1);
        let site_pages = SitePage::find().all(&seo_storage.get_db()).await.unwrap();
        assert_eq!(site_pages.len(), 1);
        println!("site_pages: {:?}", site_pages);
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
                    rule_id: "test".to_string(),
                    name: "test".to_string(),
                    plugin_name: "test".to_string(),
                    passed: true,
                    message: "test".to_string(),
                    severity: Severity::Info,
                    category: RuleCategory::Performance,
                    context: SiteCheckContext::Empty,
                }],
            }),
        };
        let test_page_results_clone = test_page_results.clone();

        let _ = seo_storage
            .insert_many_page_rule_results(site_run_id, test_page_results)
            .await
            .unwrap();

        let _ = seo_storage
            .insert_many_page_rule_results(site_run_id, test_page_results_clone)
            .await
            .unwrap();

        let page_rule_results = PageRuleResult::find()
            .all(&seo_storage.get_db())
            .await
            .unwrap();
        assert_eq!(page_rule_results.len(), 1);
        println!("page_rule_results: {:?}", page_rule_results);
    }
}
