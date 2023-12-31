use crate::{
    tasks::homer,
    templates::{Home, Login, Main, F04},
};
use askama_axum::IntoResponse;
use axum::response::Html;

// Handlers are called by the router

pub(crate) async fn home() -> Html<String> {
    Html(
        Main {
            body: Home {
                task_string: homer(),
            }
            .to_string(),
            title: "Mysteries of Code".to_string(),
        }
        .to_string(),
    )
}

pub(crate) async fn login() -> Login {
    Login {}
}

pub(crate) async fn login_process() -> String {
    "test".to_string()
}

pub(crate) async fn handle_404() -> F04 {
    F04 {}
}

pub(crate) async fn stylize(title: String, body: String) -> String {
    Main { body, title }.to_string()
}
