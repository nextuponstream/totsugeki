use log::error;
use poem::listener::{Listener, RustlsCertificate, RustlsConfig, TcpListener};
use poem::EndpointExt;
use poem::Result;
use poem::Route;
use poem::{http::Method, middleware::Cors};
use poem_openapi::{OpenApi, OpenApiService};
use std::boxed::Box;
use std::env;
use std::sync::Arc;
use totsugeki::ReadLock;
use totsugeki_api::hmac;
use totsugeki_api::persistence::inmemory::InMemoryDBAccessor;
use totsugeki_api::persistence::sqlite::Sqlite;
use totsugeki_api::persistence::DBAccessor;
use totsugeki_api::routes::bracket::BracketApi;
use totsugeki_api::routes::organiser::OrganiserApi;
use totsugeki_api::routes::test_utils::TestUtilsApi;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let testing_mode = env::var("TESTING").is_ok();
    if testing_mode {
        dotenv::from_filename(".env-test").expect("Failed to load .env-test file");
    } else {
        dotenv::dotenv().expect("Failed to load .env file");
    }

    let addr = env::var("API_ADDR").expect("API_ADDR");
    let port = env::var("API_PORT").expect("API_PORT");
    let full_addr = format!("{addr}:{port}");
    if testing_mode {
        serve_server(
            OpenApiService::new(
                (BracketApi, OrganiserApi, TestUtilsApi),
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
            )
            .server(format!("https://{full_addr}")),
            &full_addr,
        )
        .await
    } else {
        serve_server(
            OpenApiService::new(
                (BracketApi, OrganiserApi, TestUtilsApi),
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
            )
            .server(format!("https://{full_addr}")),
            &full_addr,
        )
        .await
    }
}

type Database = Box<dyn DBAccessor + Send + Sync>;

async fn serve_server<T: OpenApi + 'static>(
    api_service: OpenApiService<T, ()>,
    full_addr: &str,
) -> Result<(), std::io::Error> {
    let ui = api_service.swagger_ui();

    let db_type = env::var("API_DATABASE_TYPE").expect("API_DATABASE_TYPE");
    let db: Database = match db_type.as_str() {
        "SQLITE" => {
            let sqlite_file_path = env::var("SQLITE_FILE_PATH").expect("SQLITE_FILE_PATH");
            Box::new(Sqlite::new(&sqlite_file_path)) as Box<dyn DBAccessor + Send + Sync>
        }
        "INMEMORY" => Box::new(InMemoryDBAccessor::default()) as Box<dyn DBAccessor + Send + Sync>,
        _ => {
            error!("could not parse API_DATABASE_TYPE");
            panic!("could not parse API_DATABASE_TYPE")
        }
    };
    let db = Arc::new(ReadLock::new(db));
    db.read()
        .expect("database")
        .init()
        .expect("initialise database");

    let cors = Cors::new().allow_method(Method::GET);
    let server_key = env::var("API_KEY_PATH").expect("API_KEY_PATH");
    let server_key = std::fs::read(server_key).expect("could not read key");
    let server_key = hmac(&server_key);
    let app = Route::new()
        .nest("/", api_service)
        .nest("/swagger", ui)
        .with(cors)
        .data(db)
        .data(server_key);

    let cert_path = env::var("API_CERT_PATH").expect("API_CERT_PATH");
    let key_path = env::var("API_PRIVATE_KEY_PATH").expect("API_PRIVATE_KEY_PATH");
    let cert = std::fs::read_to_string(cert_path).expect("Could not read cert");
    let key = std::fs::read_to_string(key_path).expect("Could not read key");
    let config = RustlsConfig::new().fallback(RustlsCertificate::new().cert(cert).key(key));
    let tls = TcpListener::bind(full_addr).rustls(config);
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("error"));

    poem::Server::new(tls).run(app).await
}
