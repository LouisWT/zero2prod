use actix_web::dev::Server;
use sqlx::postgres::PgPoolOptions;
use zero2prod::startup::run;
use std::net::TcpListener;
use secrecy::ExposeSecret;
use tracing::Subscriber;
use zero2prod::configuration::Settings;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
use tracing_log::LogTracer;
use zero2prod::telemetry::{init_subscriber, get_subsciber};


// #[actix_web::main] // or #[tokio::main]
// async fn main() -> std::io::Result<()>{
//     let _ = run().await;
//     Ok(())
// }

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subsciber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    // Bubble up the io::Error if we failed to bind the address
    // Otherwise call .await on our Server
    let configuration = Settings::new().expect("Fail to load config");
    let connect_option = configuration.database.with_db();
    // let mut connection = PgPoolOptions:new().connect_timeout(std::time::Duration::from_secs(2)).connect_lazy(&connect_str.expose_secret()).expect("Fail to connect database");
    let mut connection = PgPoolOptions::new().acquire_timeout(std::time::Duration::from_secs(2)) .connect_lazy_with(connect_option);
    let address = format!("{}:{}", &configuration.application.host, &configuration.application.port);
    let tcpListener = TcpListener::bind(&address).expect(&format!("fail to bind {}", &address));
    run(tcpListener, connection)?.await
}