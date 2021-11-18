use axum::routing::post;
use axum::AddExtensionLayer;
use axum::{routing::get, Router, Server};
use routes::*;
use sqlx::PgPool;
use std::future::Future;
use std::net::TcpListener;
use telemetry::{AxumMakeSpan, AxumOnFailure, AxumOnRequest, AxumOnResponse};
use tower_http::trace::TraceLayer;

pub mod configuration;
pub mod routes;
pub mod telemetry;

/// Retrieve a server to run
///
/// Takes a TcpListerer to run server on and a PgPool for
/// for routes to connect to database. PgPool is turned
/// into a layered middleware
pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
) -> Result<impl Future<Output = hyper::Result<()>>, hyper::Error> {
    let db_layer;
    let server;
    let app;

    db_layer = AddExtensionLayer::new(db_pool);

    app = Router::new()
        .route("/health_check", get(health_check))
        .route("/user", post(add_user))
        .layer(db_layer)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(AxumMakeSpan())
                .on_request(AxumOnRequest)
                .on_response(AxumOnResponse)
                .on_body_chunk(())
                .on_eos(())
                .on_failure(AxumOnFailure),
        );

    server = Server::from_tcp(listener)?.serve(app.into_make_service());

    Ok(server)
}
