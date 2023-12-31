use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

use crate::{tasks::Task, TASKS};

#[derive(Template)]
#[template(path = "home.html", escape = "none")]
pub struct Home {
    pub task_string: String,
}

#[derive(Template)]
#[template(path = "login.html")]
pub struct Login {}

#[derive(Template)]
#[template(path = "main.html", escape = "none")]
pub struct Main {
    pub title: String,
    pub body: String,
}

#[derive(Template)]
#[template(path = "F04.html")]
pub struct F04 {}

///////////////////////////////////////////////////////////////////////////////////////

pub fn parse_md(file_name: String, file_content: String) -> Option<(String, String)> {
    if !file_content.lines().nth(0).unwrap().contains("---") {
        eprintln!("Task {} has no config", file_name);
        None
    } else {
        let mut task_str = "".to_string();
        let mut first = true;
        let mut has_end = false;
        let mut last_line_num = 0;
        for (n, line) in file_content.lines().enumerate() {
            if line.contains("---") && n != 0 {
                has_end = true;
                last_line_num = n;
                break;
            }
            if !line.starts_with("//") && n != 0 {
                if first {
                    task_str = format!("{},", line);
                    first = false;
                } else {
                    task_str = format!("{}\n{},", task_str, line);
                }
            }
        }
        if !has_end {
            eprintln!(
                "Task {} has no config end \"---\" at the end missing",
                file_name
            );
            return None;
        }
        let md_str: String = file_content.lines().skip(last_line_num).collect();

        println!("{}", task_str);
        Some((task_str, md_str))
    }
}

fn add_task(title: String, description: String, url: String, release_date_ymd: (i32, u32, u32)) {
    unsafe {
        TASKS
            .tasks
            .push(Task::new(title, description, url, release_date_ymd))
    }
}

////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    #[test]
    fn test_md_parsing() {
        use super::parse_md;
        use std::fs;

        let md_content = fs::read_to_string("res/articles/example.md").unwrap();
        let solution = "\"title\": \"Task Title\",
\"description\": \"Description for Title\",
\"release_date\": \"2006-02-16T00:00:00+01:00\""
            .to_string();
        let (task_str, _) = parse_md("Test".to_string(), md_content).unwrap();
        assert_eq!(task_str, solution);
        assert_eq!(
            None,
            parse_md("Test2".to_string(), "aaaaaa\nasdadad".to_string())
        );
    }
}
