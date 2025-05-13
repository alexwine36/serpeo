use entities::{Run, Site};
use std::sync::Arc;
use tokio::sync::Mutex;
pub mod entities;
pub mod migrations;
pub use welds::{
    connections::{Client, any::AnyClient, sqlite::SqliteClient},
    state::DbState,
};
// pub mod migrations;
#[derive(Clone)]
pub struct SeoStorage {
    connection_string: String,
    client: Arc<Mutex<AnyClient>>,
}

impl SeoStorage {
    pub async fn new(connection_string: &str) -> Result<Self, String> {
        let client = welds::connections::connect(connection_string)
            .await
            .map_err(|e| e.to_string())?;
        Ok(Self {
            connection_string: connection_string.to_string(),
            client: Arc::new(Mutex::new(client)),
        })
    }
    async fn migrate(&self) -> Result<(), String> {
        let migrations = migrations::get_migrations();
        for migration in migrations {
            let client = self.client.lock().await;
            client
                .execute(&migration.schema, &[])
                .await
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub async fn upsert_site(&self, url: &str) -> Result<i32, String> {
        let mut site = Site::new();
        site.url = url.to_string();
        let client = self.client.lock().await;
        let sites = Site::all()
            .where_col(|p| p.url.equal(url))
            .limit(1)
            .run(client.as_ref())
            .await
            .map_err(|e| e.to_string())?;
        if !sites.is_empty() {
            return Ok(sites[0].id);
        }
        site.save(client.as_ref())
            .await
            .map_err(|e| e.to_string())?;
        Ok(site.id)
    }

    pub async fn create_run(&self, url: &str) -> Result<i32, String> {
        let site_id = self.upsert_site(url).await?;
        let client = self.client.lock().await;
        let mut run = Run::new();
        run.site_id = site_id;
        run.save(client.as_ref()).await.map_err(|e| e.to_string())?;
        Ok(run.id)
    }

    pub async fn get_sites(&self) -> Result<Vec<Site>, String> {
        let client = self.client.lock().await;
        let sites = Site::all()
            .run(client.as_ref())
            .await
            .map(|s| s.into_iter().map(|s| s.into_inner()).collect())
            .map_err(|e| e.to_string())?;
        Ok(sites)
    }

    pub async fn get_runs(&self) -> Result<Vec<Run>, String> {
        let client = self.client.lock().await;
        let runs = Run::all()
            .run(client.as_ref())
            .await
            .map(|s| s.into_iter().map(|s| s.into_inner()).collect())
            .map_err(|e| e.to_string())?;
        Ok(runs)
    }

    // pub async fn create_run(&self, url: &str) -> Result<i32, String> {
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_migrate() {
        let storage = SeoStorage::new("sqlite::memory:").await.unwrap();
        storage.migrate().await.unwrap();
    }

    #[tokio::test]
    async fn test_upsert_site() {
        let storage = SeoStorage::new("sqlite::memory:").await.unwrap();
        storage.migrate().await.unwrap();
        let site_id = storage.upsert_site("https://www.google.com").await.unwrap();
        assert!(site_id > 0);
        let site_id2 = storage.upsert_site("https://www.google.com").await.unwrap();
        assert_eq!(site_id, site_id2);
    }

    #[tokio::test]
    async fn test_create_run() {
        let storage = SeoStorage::new("sqlite::memory:").await.unwrap();
        storage.migrate().await.unwrap();
        let run_id = storage.create_run("https://www.google.com").await.unwrap();
        assert!(run_id > 0);
        let runs = storage.get_runs().await.unwrap();
        println!("runs: {:?}", runs);
        assert!(!runs.is_empty());
        let created_run = &runs[0];
        assert_eq!(created_run.id, run_id);
        let site_id = storage.upsert_site("https://www.google.com").await.unwrap();
        println!("site_id: {}", site_id);
        let sites = storage.get_sites().await.unwrap();
        println!("sites: {:?}", sites);

        // assert!(false);
    }
}
