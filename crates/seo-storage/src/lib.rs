use entities::prelude::{SitePage, SiteRun};
use entities::{site, site_page, site_run};
use enums::db_link_type::DbLinkType;
use enums::site_run_status::SiteRunStatus;
use migration::{Migrator, MigratorTrait, OnConflict};
use sea_orm::*;
use sea_orm::{Database, DbErr};
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

    pub async fn new(db_url: &str) -> Self {
        let db = Database::connect(db_url).await.unwrap();
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

    // Database interaction

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
}

#[cfg(test)]
mod tests {

    use migration::SchemaManager;

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
}
