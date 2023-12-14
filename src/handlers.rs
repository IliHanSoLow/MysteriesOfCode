use crate::{
    tasks::homer,
    templates::{BlumaTest, Home, Login},
};
use askama_axum::IntoResponse;

pub(crate) async fn home() -> Home {
    Home {
        task_string: homer(),
    }
}

pub(crate) async fn login() -> Login {
    Login {}
}

pub(crate) async fn login_process() -> String {
    "test".to_string()
}

pub(crate) async fn bluma_test() -> BlumaTest {
    BlumaTest {}
}
