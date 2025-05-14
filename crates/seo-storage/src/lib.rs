use entities::site;
use migration::{Migrator, MigratorTrait, OnConflict};
use sea_orm::*;
use sea_orm::{Database, DbErr};

pub mod entities;
use crate::entities::prelude::Site;

const DATABASE_URL: &str = "sqlite::memory:";

pub struct SeoStorage {
    db: DatabaseConnection,
}

impl SeoStorage {
    // Utilities
    pub async fn new_with_default() -> Self {
        let db = Database::connect(DATABASE_URL).await.unwrap();
        SeoStorage { db }
    }

    pub fn get_db(&self) -> DatabaseConnection {
        self.db.clone()
    }

    pub async fn migrate_up(&self) -> Result<(), DbErr> {
        Migrator::up(&self.db, None).await.unwrap();

        Ok(())
    }

    // Database interaction
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
        let seo_storage = SeoStorage::new_with_default().await;
        let db = seo_storage.get_db();
        let _ = seo_storage.migrate_up().await;

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

        assert!(false);
    }
}
