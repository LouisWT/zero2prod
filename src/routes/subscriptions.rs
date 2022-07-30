use actix_web::{web, App, HttpResponse, HttpServer};
use actix_web::dev::Server;
use sqlx::{PgPool};
use tokio::net::TcpListener;
use uuid::Uuid;
use chrono::Utc;
use tracing::{Instrument, Subscriber};

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber.",
    skip(form, connection),
    fields(
        subscriber_email =  %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(form: web::Form<FormData>, connection: web::Data<PgPool>) -> HttpResponse {
    // let request_id = Uuid::new_v4();
    // // Spans, like logs, have an associated level // `info_span` creates a span at the info-level
    // let request_span = tracing::info_span!(
    //
    //     %request_id,
    //     subscriber_email = %form.email,
    //     subscriber_name= %form.name
    // );
    // // Using `enter` in an async function is a recipe for disaster!
    // // Bear with me for now, but don't do this at home.
    // // See the following section on `Instrumenting Futures`
    // let _request_span_guard = request_span.enter();
    // let query_span = tracing::info_span!("Saving new subscriber details in the database");

    match insert_subscriber(&form, &connection).await {
        Ok(_) => {
            HttpResponse::Ok().finish()
        },
        Err(e) => {
            // {:?} fmt::Debug 可以输出尽可能详细的错误信息
            // {} fmt::Display 可以输出比较好看的错误信息，方便给终端用户展示
            HttpResponse::InternalServerError().finish()
        }

    }
}

// web::FormData 和 Data 都实现了 Deref，所以可以直接把 &form 当作 &FormData 使用
#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(form, connection),
)]
async fn insert_subscriber(form: &FormData, connection: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscription (id, email, name, subscribe_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now().naive_utc()
    )
        .execute(connection)
        .await.map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
        })?;
    Ok(())
}