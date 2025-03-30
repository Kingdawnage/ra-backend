use std::{env, sync::Arc};

use api::create_api;
use axum::{http::{header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE}, HeaderValue, Method}, routing::get, Json};
use config::Config;
use dotenvy::dotenv;
use services::database::DBClient;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing_subscriber::filter::LevelFilter;

mod models;
mod utils;
mod config;
mod services;
mod routes;
mod api;

#[derive(Debug, Clone)]
pub struct AppState {
    pub env: Config,
    pub db_client: DBClient,
    pub http_client: reqwest::Client,
}

pub async fn run()
{
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .init();

    dotenv().ok();

    let config = Config::init();

    let pool = match PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
    {
        Ok(pool) => {
            println!("Connection to the database is successful");
            pool
        }

        Err(err) => {
            println!("Failed to connect to the database: {:?}", err);
            std::process::exit(1);
        }
    };

    let frontend_url = env::var("FRONTEND_URL").unwrap_or("http://localhost:3000".to_string());
    let _allowed_origins = frontend_url.split(",").collect::<Vec<_>>();
    
    let cors = CorsLayer::new()
        .allow_origin(frontend_url.parse::<HeaderValue>().unwrap())
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE])
        .allow_credentials(true)
        .allow_methods([Method::GET, Method::POST, Method::PUT]);

    let db_client = DBClient::new(pool);
    let http_client = reqwest::Client::new();
    let app_state = AppState {
        env: config.clone(),
        db_client,
        http_client,
    };

    let app = create_api(Arc::new(app_state.clone()))
        .route("/", get(|| async {Json("Hello, World!")}))
        .route("/health", get(|| async {Json("OK")}))
        .layer(cors.clone());

    println!("Server is running on 0.0.0.0:{}", config.port);

    let listener = TcpListener::bind(format!("0.0.0.0:{}", config.port))
        .await
        .unwrap();

    println!("Listening on: {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}