use anyhow::Context;
use askama::Template;
use axum::{
    extract::Host,
    handler::HandlerWithoutStateExt,
    http::{header::CONTENT_TYPE, HeaderValue, Method},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
    BoxError, Router,
};
use axum_server::tls_rustls::RustlsConfig;
use handlers::*;
use hyper::{StatusCode, Uri};
use std::{
    fs::{read_to_string, File},
    net::SocketAddr,
    path::PathBuf,
};
use tasks::*;
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::login::login_process;

mod handlers;
mod login;
mod tasks;
mod templates;

static mut TASKS: Tasks = Tasks { tasks: Vec::new() };

// #[allow(dead_code)]
// #[derive(Clone, Copy)]
// struct Ports {
//     http: u16,
//     https: u16,
// }

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mysteries_of_code=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("starting...");
    let rdr = File::open("res/tasks.json")?;
    unsafe {
        TASKS = serde_json::from_reader(rdr)?;
    }

    info!("{:?}", unsafe { TASKS.tasks() });

    info!("initializing router and assets");

    // let ports = Ports {
    //     http: 7878,
    //     https: 80,
    // };

    // let config = RustlsConfig::from_pem_file(
    //     PathBuf::from("mysteriesofcode.pem"),
    //     PathBuf::from("mysteriesofcode-key.pem"),
    // )
    // .await
    // .unwrap();

    let assets_path = std::env::current_dir()?;

    let api_router: Router = Router::new()
        .route("/hello", get(say_hello))
        .route("/login", post(login_process));

    let app = Router::new()
        .route("/", get(home))
        .route("/login", get(login))
        .route("/bluma_test", get(bluma_test))
        .nest("/api", api_router)
        .nest_service(
            "/assets",
            ServeDir::new(format!("{}/assets", assets_path.to_str().unwrap())),
        );

    info!("{}", format!("{}/assets", assets_path.to_str().unwrap()));

    let addr = SocketAddr::from(([127, 0, 0, 1], 80));
    // let addr = SocketAddr::from(([127, 187, 187, 187], ports.https));

    info!("router initialized, not listening on port {}", 80);
    info!("http://{addr}");

    let listenser = tokio::net::TcpListener::bind(&addr)
        .await
        .context("error while starting API server")?;

    axum::serve(listenser, app).await?;

    // let ax_server = axum_server::bind_rustls(addr, config);
    // axum::serve(ax_server, app);
    // tokio::spawn(redirect_http_to_https(ports));

    Ok(())
}

async fn say_hello() -> &'static str {
    "Hello!"
}

// #[allow(dead_code)]
// async fn redirect_http_to_https(ports: Ports) {
//     fn make_https(host: String, uri: Uri, ports: Ports) -> Result<Uri, BoxError> {
//         let mut parts = uri.into_parts();

//         parts.scheme = Some(axum::http::uri::Scheme::HTTPS);

//         if parts.path_and_query.is_none() {
//             parts.path_and_query = Some("/".parse().unwrap());
//         }

//         let https_host = host.replace(&ports.http.to_string(), &ports.https.to_string());
//         parts.authority = Some(https_host.parse()?);

//         Ok(Uri::from_parts(parts)?)
//     }

//     let redirect = move |Host(host): Host, uri: Uri| async move {
//         match make_https(host, uri, ports) {
//             Ok(uri) => Ok(Redirect::permanent(&uri.to_string())),
//             Err(error) => {
//                 tracing::warn!(%error, "failed to convert URI to HTTPS");
//                 Err(StatusCode::BAD_REQUEST)
//             }
//         }
//     };

//     let addr = SocketAddr::from(([127, 187, 187, 187], ports.http));
//     let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
//     tracing::debug!("listening on {}", listener.local_addr().unwrap());
//     axum::serve(listener, redirect.into_make_service())
//         .await
//         .unwrap();
// }
