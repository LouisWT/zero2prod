//! tests/health_check.rs

use std::net::{Shutdown, TcpListener};
use actix_web::dev::Server;
use zero2prod::startup::run;
use sqlx::{PgConnection, Connection, PgPool, Executor};

use std::{thread, time};
use zero2prod::configuration::{DatabaseSettings, Settings};
use uuid::Uuid;
use zero2prod::telemetry::{get_subsciber, init_subscriber};
use once_cell::sync::Lazy;
use secrecy::ExposeSecret;

static TRACING: Lazy<()> = Lazy::new(|| {
    let env = std::env::var("TEST_LOG");
    if env.is_ok() {
        let subscriber = get_subsciber("test".into(), "debug".into(), std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subsciber("test".into(), "debug".into(), std::io::sink);
        init_subscriber(subscriber);
    }

});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
// Create database
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str()) .await
        .expect("Failed to create database.");
    // Migrate database
    let connection_pool = PgPool::connect_with(config.with_db()) .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");
    connection_pool
}

// spawn app 不能是异步函数，否则要阻塞执行才能将 server 起来，这样下面的代码就没法执行了
async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let mut settings = Settings::new().expect("Fail to load config");
    settings.database.database_name = Uuid::new_v4().to_string();
    let mut listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    // We retrieve the port assigned to us by the OS
    let port = listener.local_addr().unwrap().port();
    let query_address = format!("http://127.0.0.1:{}", port);

    let  connection = configure_database(&settings.database).await;
    let server = run(listener, connection.clone()).unwrap();
    let _ = tokio::spawn(server);

    TestApp {
        address: query_address,
        db_pool: connection
    }
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {

    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");
    assert!(response.status().is_success());

    let saved = sqlx::query!("SELECT email, name FROM subscription").fetch_one(&app.db_pool).await.expect("fail to fetch database item");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email")
    ];
    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
            .post(&format!("{}/subscriptions", &app.address)) .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");
        // Assert
        assert_eq!( 400,
            response.status().as_u16(),
            // Additional customised error message on test failure
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}