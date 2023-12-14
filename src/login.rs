use anyhow::Result;
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::Html,
    Form, Json,
};
use serde::Deserialize;
use sqlx::postgres::PgPool;
use tracing::info;

#[derive(Deserialize)]
pub struct Login {
    email: String,
    password: String,
}

pub(crate) async fn login_process(Form(login): Form<Login>) -> String {
    println!("Called login_process");
    info!(
        "Called login_process with {:?}, {:?}",
        login.email, login.password
    );

    let output = format!("{}", check_user_in_db(&login.email, &login.password).await);
    info!(output);

    output
}

async fn check_user_in_db(user_mail: &str, user_password: &str) -> String {
    let pool = PgPool::connect("postgres://postgres:12345678@localhost:5432")
        .await
        .unwrap();

    let result = sqlx::query!(
        "SELECT EXISTS(
            SELECT * FROM user_data WHERE username = $1 AND password = crypt($2, password)
        )",
        user_mail,
        user_password,
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    format!("{:?}", result)
}
