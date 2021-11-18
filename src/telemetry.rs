use std::borrow::Cow;

use hyper::{Method, Request, Response, Version};
use tower_http::trace::{MakeSpan, OnFailure, OnRequest, OnResponse};
use tracing::Level;
use tracing::{subscriber::set_global_default, Span, Subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{fmt::MakeWriter, layer::SubscriberExt, EnvFilter, Registry};
use uuid::Uuid;

/// Creates a layered subscriber
///
/// Layers:
///  - Env-layer: The subscriber has name, and outputs to sink at
/// env_filter level or above (if RUST_LOG not set)
///  - Bunyan: formats in json, then bunyan
pub fn get_subscriber(
    name: String,
    env_filter: String,
    sink: impl MakeWriter + Send + Sync + 'static,
) -> impl Subscriber + Send + Sync {
    let formatting_layer;

    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));

    // Formats information in the Bunyan Format. Relies on JsonStorageLayer for field access
    formatting_layer = BunyanFormattingLayer::new(name, sink);

    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

/// Register subscriber as global default to process span data
///
/// Only call once
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber");
}

#[derive(Clone)]
pub struct AxumMakeSpan();

impl<B> MakeSpan<B> for AxumMakeSpan {
    fn make_span(&mut self, req: &Request<B>) -> Span {
        let user_agent;
        let http_method;
        let target;
        let span;

        user_agent = req
            .headers()
            .get("User-Agent")
            .map(|h| h.to_str().unwrap_or(""))
            .unwrap_or("");

        http_method = http_method_str(req.method());
        target = req.uri().path_and_query().map(|p| p.as_str()).unwrap_or("");

        span = tracing::info_span!(
            "HTTP Request",
            http.method = %http_method,
            http.user_agent = %user_agent,
            http.target = %target,
            http.status_code = tracing::field::Empty,
            request_id = %Uuid::new_v4(),
            trace_id = tracing::field::Empty,
            exception.message = tracing::field::Empty,
            exception.details = tracing::field::Empty
        );

        span
    }
}

#[inline]
pub fn http_method_str(method: &Method) -> Cow<'static, str> {
    match method {
        &Method::OPTIONS => "OPTIONS".into(),
        &Method::GET => "GET".into(),
        &Method::POST => "POST".into(),
        &Method::PUT => "PUT".into(),
        &Method::DELETE => "DELETE".into(),
        &Method::HEAD => "HEAD".into(),
        &Method::TRACE => "TRACE".into(),
        &Method::CONNECT => "CONNECT".into(),
        &Method::PATCH => "PATCH".into(),
        other => other.to_string().into(),
    }
}

#[inline]
pub fn http_flavor(version: Version) -> Cow<'static, str> {
    match version {
        Version::HTTP_09 => "0.9".into(),
        Version::HTTP_10 => "1.0".into(),
        Version::HTTP_11 => "1.1".into(),
        Version::HTTP_2 => "2.0".into(),
        Version::HTTP_3 => "3.0".into(),
        other => format!("{:?}", other).into(),
    }
}

#[derive(Clone)]
pub struct AxumOnRequest;

impl AxumOnRequest {
    pub fn new() -> Self {
        AxumOnRequest
    }
}

impl<B> OnRequest<B> for AxumOnRequest {
    fn on_request(&mut self, _request: &Request<B>, _span: &Span) {
        tracing::event!(Level::INFO, "started processing request",)
    }
}

#[derive(Clone)]
pub struct AxumOnResponse;

impl<B> OnResponse<B> for AxumOnResponse {
    fn on_response(self, resp: &Response<B>, latency: std::time::Duration, span: &Span) {
        span.record("http.status_code", &tracing::field::display(resp.status()));
        tracing::event!(
            Level::INFO,
            latency = format_args!("{} ms", latency.as_millis()),
            "finished processing request",
        );
    }
}

#[derive(Clone)]
pub struct AxumOnFailure;

impl<B> OnFailure<B> for AxumOnFailure
where
    B: std::fmt::Display,
    B: std::fmt::Debug,
{
    fn on_failure(
        &mut self,
        failure_classification: B,
        latency: std::time::Duration,
        _span: &Span,
    ) {
        tracing::event!(
            Level::ERROR,
            classification = tracing::field::display(&failure_classification),
            latency = format_args!("{} ms", latency.as_millis()),
            "response failed",
        )
    }
}
