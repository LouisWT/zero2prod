use actix_web::{web, App, HttpResponse, HttpServer};
use actix_web::dev::Server;
use tokio::net::TcpListener;

pub async fn health_check() -> HttpResponse { HttpResponse::Ok().finish()
}