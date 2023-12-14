use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

#[derive(Template)]
#[template(path = "home.html", escape = "none")]
pub struct Home {
    pub task_string: String,
}

#[derive(Template)]
#[template(path = "login.html")]
pub struct Login {}

#[derive(Template)]
#[template(path = "main.html")]
pub struct BlumaTest {}
