use serde::{Deserialize, Serialize};
use specta::Type;
use sqlx::types::chrono::{DateTime, Utc};
use welds::{Syntax, WeldsError, prelude::*};

#[derive(Debug, WeldsModel, Serialize, Deserialize, Type)]
#[welds(table = "Site")]
#[welds(HasMany(run, Run, "site_id"))]
#[welds(BeforeCreate(before_create_site))]
pub struct Site {
    #[welds(primary_key)]
    pub id: i32,
    pub name: String,
    pub url: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, WeldsModel)]
#[welds(table = "Run")]
#[welds(HasMany(page_runs, PageRun, "run_id"))]
#[welds(BeforeCreate(before_create_run))]
pub struct Run {
    #[welds(primary_key)]
    pub id: i32,
    #[welds(rename = "siteId")]
    pub site_id: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, WeldsModel)]
#[welds(table = "PageRun")]
#[welds(HasMany(rule_results, RuleResult, "page_run_id"))]
#[welds(BeforeCreate(before_create_page_run))]
pub struct PageRun {
    #[welds(primary_key)]
    pub id: i32,
    pub run_id: i32,
    pub url: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, WeldsModel)]
#[welds(table = "RuleResult")]
#[welds(BeforeCreate(before_create_rule_result))]
pub struct RuleResult {
    #[welds(primary_key)]
    pub id: i32,
    pub page_run_id: i32,
    pub rule_id: String,
    pub rule_type: String,
    pub passed: bool,
    // pub context: JsonValue,
    pub created_at: DateTime<Utc>,
}

fn before_create_site(model: &mut Site) -> welds::errors::Result<()> {
    model.created_at = Utc::now();
    Ok(())
}

fn before_create_run(model: &mut Run) -> welds::errors::Result<()> {
    model.created_at = Utc::now();
    Ok(())
}

fn before_create_page_run(model: &mut PageRun) -> welds::errors::Result<()> {
    model.created_at = Utc::now();
    Ok(())
}

fn before_create_rule_result(model: &mut RuleResult) -> welds::errors::Result<()> {
    model.created_at = Utc::now();
    Ok(())
}
