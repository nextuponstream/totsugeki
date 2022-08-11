use poem::listener::{Listener, RustlsCertificate, RustlsConfig, TcpListener};
use poem::Result;
use poem::Route;
use poem_openapi::{OpenApi, OpenApiService};
use std::env;
use totsugeki_api::routes::bracket::Api as BracketApi;
use totsugeki_api::routes::health_check::Api as HealthcheckApi;
use totsugeki_api::routes::join::Api as JoinApi;
use totsugeki_api::routes::organiser::Api as OrganiserApi;
use totsugeki_api::routes::service::Api as ServiceApi;
use totsugeki_api::{oai_test_service, route_with_data, DatabaseType};
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::BunyanFormattingLayer;
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv::dotenv().expect("Failed to load .env file");
    let mode_path = env::var("API_MODE_PATH").expect("API_MODE_PATH");
    let mode = std::fs::read_to_string(mode_path).expect("could not read mode of API");
    let testing_mode = mode == "testing";

    let addr = env::var("API_ADDR").expect("API_ADDR");
    let port = env::var("API_PORT").expect("API_PORT");
    let full_addr = format!("{addr}:{port}");
    let db_type = env::var("API_DATABASE_TYPE").expect("API_DATABASE_TYPE");
    let db_type = db_type.parse::<DatabaseType>().expect("database type");
    if testing_mode {
        serve_server(
            oai_test_service().server(format!("https://{full_addr}")),
            &full_addr,
            db_type,
        )
        .await
    } else {
        serve_server(
            OpenApiService::new(
                (
                    BracketApi,
                    OrganiserApi,
                    ServiceApi,
                    JoinApi,
                    HealthcheckApi,
                ),
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
            )
            .server(format!("https://{full_addr}")),
            &full_addr,
            db_type,
        )
        .await
    }
}

async fn serve_server<T: OpenApi + 'static>(
    api_service: OpenApiService<T, ()>,
    full_addr: &str,
    db_type: DatabaseType,
) -> Result<(), std::io::Error> {
    let server_key = env::var("API_KEY_PATH").expect("API_KEY_PATH");
    let server_key = std::fs::read(server_key).expect("could not read key");

    let ui = api_service.swagger_ui();
    let app = Route::new().nest("/", api_service);
    let app = app.nest("/swagger", ui);
    let app = route_with_data(app, db_type, &server_key);

    let cert_path = env::var("API_CERT_PATH").expect("API_CERT_PATH");
    let key_path = env::var("API_PRIVATE_KEY_PATH").expect("API_PRIVATE_KEY_PATH");
    let cert = std::fs::read_to_string(cert_path).expect("Could not read cert");
    let key = std::fs::read_to_string(key_path).expect("Could not read key");
    let config = RustlsConfig::new().fallback(RustlsCertificate::new().cert(cert).key(key));
    let tls = TcpListener::bind(full_addr).rustls(config);

    LogTracer::init().expect("Failed to set logger");
    let formatting_layer = BunyanFormattingLayer::new(
        "totsugeki-api".into(),
        // Output the formatted spans to stdout.
        std::io::stdout,
    );
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let subscriber = Registry::default().with(env_filter).with(formatting_layer);
    set_global_default(subscriber).expect("Failed to set subscriber.");
    // env_logger::init_from_env(env_logger::Env::new().default_filter_or("error")); // TODO remove

    poem::Server::new(tls).run(app).await
}
