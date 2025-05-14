pub use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250514_154203_create_site_table::Migration),
            Box::new(m20250514_171121_create_site_run_table::Migration),
        ]
    }
}
mod m20250514_154203_create_site_table;
mod m20250514_171121_create_site_run_table;
