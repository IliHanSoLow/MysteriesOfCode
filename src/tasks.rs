use axum::response::Html;
use chrono::{DateTime, Local, LocalResult, TimeZone, Utc};
use chrono_tz::{Etc::UTC, Europe::Berlin, Tz};
use serde::{Deserialize, Serialize};
use tracing::info;
use url::Url;

use crate::TASKS;

#[derive(Serialize, Deserialize)]
pub(crate) struct Tasks {
    pub tasks: Vec<Task>,
}

impl Tasks {
    pub(crate) fn tasks(&self) -> &Vec<Task> {
        &self.tasks
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Task {
    title: String,
    description: String,
    url: String,
    release_date: DateTime<Utc>,
}

impl Task {
    pub(crate) fn new(
        title: String,
        description: String,
        url: String,
        release_date_ymd: (i32, u32, u32),
    ) -> Self {
        let (y, m, d) = release_date_ymd;
        let release_date = Berlin
            .with_ymd_and_hms(y, m, d, 0, 0, 0)
            .unwrap()
            .with_timezone(&Utc);
        Self {
            title,
            description,
            url,
            release_date,
        }
    }
}

pub(crate) fn homer() -> String {
    let mut table: String = "<table>".to_string();
    for i in unsafe { TASKS.tasks() } {
        if i.release_date <= Utc::now() {
            table = format!(
                "{}\n <tr><th><a href=\"{}\">{}</a></th><th>{}</th></tr>",
                table, i.url, i.title, i.description,
            );
        }
    }
    info!(table);
    table
}
