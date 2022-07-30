use std::net::{SocketAddr, ToSocketAddrs};
use actix_web::{web, App, HttpResponse, HttpServer};
use actix_web::dev::Server;
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

use crate::routes::*;

// We need to mark `run` as public.
// It is no longer a binary entrypoint, therefore we can mark it as async // without having to use any proc-macro incantation.
pub fn run(listener: TcpListener, connection: PgPool) -> Result<Server, std::io::Error> {
    let wrapConnection = web::Data::new(connection);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(wrapConnection.clone())
    })
        .listen(listener)?
        .run();
    Ok(server)
}