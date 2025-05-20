use chrono::DateTime;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::entities::site;
use crate::entities::site_run;
pub fn get_sites_with_site_runs(
    data: Vec<(site::Model, Vec<site_run::Model>)>,
) -> Vec<SiteWithSiteRuns> {
    data.into_iter()
        .map(|(site, site_runs)| SiteWithSiteRuns {
            site,
            last_site_run_at: site_runs.last().unwrap().created_at,
            site_runs,
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub struct SiteWithSiteRuns {
    pub site: site::Model,
    pub last_site_run_at: DateTime<Utc>,
    pub site_runs: Vec<site_run::Model>,
}
