pub use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250514_154203_create_site_table::Migration),
            Box::new(m20250514_171121_create_site_run_table::Migration),
            Box::new(m20250514_211317_create_site_page_table::Migration),
            Box::new(m20250514_214902_create_page_rule_result_table::Migration),
            Box::new(m20250516_171758_create_plugin_rule_table::Migration),
            Box::new(m20250516_193257_update_page_rule_result::Migration),
        ]
    }
}
mod m20250514_154203_create_site_table;
mod m20250514_171121_create_site_run_table;
mod m20250514_211317_create_site_page_table;
mod m20250514_214902_create_page_rule_result_table;
mod m20250516_171758_create_plugin_rule_table;
mod m20250516_193257_update_page_rule_result;
