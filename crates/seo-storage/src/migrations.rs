use sqlx::migrate::{Migration, MigrationType};

// pub const MIGRATIONS: Vec<Migration> = vec![Migration::new(
//     1,
//     "create_initial_tables".into(),
//     MigrationType::Simple,
//     include_str!(
//         "../../../packages/prisma/prisma/migrations/20250513041938_initial_create_db/migration.sql"
//     )
//     .into(),
//     false,
// )];

pub struct MigrationCode {
    pub version: i64,
    pub description: String,
    pub schema: String,
}

pub fn get_migrations() -> Vec<MigrationCode> {
    vec![MigrationCode {
        version: 1,
        description: "create_initial_tables".into(),
        schema: include_str!(
            "../../../packages/prisma/prisma/migrations/20250513041938_initial_create_db/migration.sql"
        )
        .into(),
    }]
}

//     migration_type: MigrationType::Simple,
//     checksum: Cow::Borrowed(b"1234567890"),
//     no_tx: false,
