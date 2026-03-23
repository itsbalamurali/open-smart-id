use opentelemetry::global;
use opentelemetry_sdk::trace::SdkTracerProvider;
use poem::middleware::{OpenTelemetryMetrics, OpenTelemetryTracing, RequestId};
use poem::{EndpointExt, Server, listener::TcpListener};
use poem_openapi::OpenApiService;
use sea_orm::DatabaseConnection;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod api;
mod db;
mod models;
mod services;

use api::{AdminApi, AppApi, AuthenticationApi, InternalApi, SessionApi, SignatureApi};
use services::{CertificateService, NotificationService, SessionNotifier};

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub certificate: CertificateService,
    pub notifier: SessionNotifier,
    pub notification: Option<NotificationService>,
}

fn init_tracing() -> Option<SdkTracerProvider> {
    let provider = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .build()
        .ok()
        .map(|exporter| {
            let provider = SdkTracerProvider::builder()
                .with_batch_exporter(exporter)
                .build();
            global::set_tracer_provider(provider.clone());
            provider
        });

    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,sqlx=warn")),
        )
        .with(tracing_subscriber::fmt::layer().compact())
        .with(provider.as_ref().map(|_| tracing_opentelemetry::layer()))
        .init();

    provider
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let _provider = init_tracing();

    let db = db::connect("sqlite://smartid.db?mode=rwc")
        .await
        .expect("failed to connect to database");

    let state = AppState {
        db,
        certificate: CertificateService::new().expect("failed to initialize CA"),
        notifier: SessionNotifier::new(),
        notification: NotificationService::from_env(),
    };

    let api_service = OpenApiService::new(
        (
            AuthenticationApi,
            SignatureApi,
            SessionApi,
            InternalApi,
            AppApi,
            AdminApi,
        ),
        "Smart-ID API",
        "3.1",
    )
    .server("http://localhost:3000");

    let tracer = global::tracer("smartid-api");

    let swagger = api_service.swagger_ui();
    let app = poem::Route::new()
        .nest("/", api_service)
        .nest("/docs", swagger)
        .with(RequestId::new())
        .with(OpenTelemetryTracing::new(tracer))
        .with(OpenTelemetryMetrics::new())
        .data(state);

    tracing::info!("Smart-ID API running on http://localhost:3000");
    tracing::info!("Swagger UI at http://localhost:3000/docs");

    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await
}
