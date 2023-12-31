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
    fs::{read_dir, read_to_string, File, OpenOptions},
    io::{Read, Write},
    net::SocketAddr,
    path::PathBuf,
};
use tasks::*;
use templates::parse_md;
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
    //NOTE Registering Debuging pipeline
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mysteries_of_code=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("starting...");

    //NOTE search trough articles and add to tasks.json
    let tasks_json = File::create("res/tasks.json").unwrap();
    tasks_json.set_len(0).unwrap();
    let mut tasks_json = OpenOptions::new()
        .append(true)
        .open("res/tasks.json")
        .unwrap();
    let paths = read_dir("./res/articles").expect("No res/articles folder (or maybe no content)");
    let json_start = "{\"tasks\":[\n";
    tasks_json.write_all(json_start.as_bytes()).unwrap();
    for path in paths {
        let path = path.unwrap();
        let file_name = path.file_name();
        let file_content = std::fs::read_to_string(path.path().display().to_string()).unwrap();
        let (json_str, md_str) = parse_md(file_name.clone().into_string().unwrap(), file_content)
            .expect("Some problems with the config of a markdown file");
        tasks_json.write("{".as_bytes()).unwrap();
        tasks_json
            .write(format!("\n{}", json_str).as_bytes())
            .expect("Writing to tasks.json failed");

        tasks_json
            .write(
                format!(
                    "\"url\": \"tasks/{}\",\n",
                    file_name.clone().into_string().unwrap()
                )
                .as_bytes(),
            )
            .unwrap();

        remove_trailing_coma("res/tasks.json");
        tasks_json.write("},\n".as_bytes()).unwrap();

        //NOTE add to html parsed articles
        let mut file = File::create(format!(
            "parsed_articles/{}",
            file_name.clone().into_string().unwrap()
        ))
        .unwrap();
        let parsed_content = stylize(
            file_name.clone().into_string().unwrap(),
            markdown::to_html(&md_str),
        )
        .await;
        file.write_all(parsed_content.as_bytes()).unwrap();
    }
    tasks_json.write("]}".as_bytes()).unwrap();

    remove_trailing_coma("res/tasks.json");

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

    let current_path = std::env::current_dir()?;

    let api_router: Router = Router::new()
        .route("/hello", get(say_hello))
        .route("/login", post(login_process));

    let app = Router::new()
        .fallback(handle_404)
        .route("/", get(home))
        .route("/login", get(login))
        .nest("/api", api_router)
        .nest_service(
            "/tasks",
            ServeDir::new(format!(
                "{}/parsed_articels",
                current_path.to_str().unwrap()
            )),
        )
        .nest_service(
            "/assets",
            ServeDir::new(format!("{}/assets", current_path.to_str().unwrap())),
        )
        .nest_service(
            "/raw",
            ServeDir::new(format!("{}/res/raw", current_path.to_str().unwrap())),
        );

    info!("{}", format!("{}/assets", current_path.to_str().unwrap()));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3213));
    // let addr = SocketAddr::from(([127, 187, 187, 187], ports.https));

    info!("router initialized, now listening on port {}", 3213);
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

fn remove_trailing_coma(file_name: &str) {
    let mut tasks_json = File::open(file_name).unwrap();
    let mut json_content = String::new();
    tasks_json.read_to_string(&mut json_content).unwrap();
    if let Some(last_camma_pos) = json_content.rfind(',') {
        json_content.remove(last_camma_pos);
        let mut tasks_json = File::create(file_name).unwrap();
        tasks_json.set_len(0).unwrap();
        tasks_json.write_all(json_content.as_bytes()).unwrap();
        info!("Successfully parsed md files to json")
    }
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
