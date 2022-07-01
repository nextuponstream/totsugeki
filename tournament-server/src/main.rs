use poem::listener::{Listener, RustlsCertificate, RustlsConfig, TcpListener};
use poem::EndpointExt;
use poem::{http::Method, middleware::Cors};
use poem::{Result, Route};
use poem_openapi::OpenApiService;
use std::boxed::Box;
use std::env;
use std::sync::{Arc, RwLock};
use tournament_server::persistence::inmemory::InMemory;
use tournament_server::persistence::sqlite::Sqlite;
use tournament_server::persistence::Database;
use tournament_server::routes::bracket::Api;

type SharedDb = Box<dyn Database + Send + Sync>;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("error"));

    match env::var("TESTING") {
        Ok(_) => dotenv::from_filename(".env-test").expect("Failed to load .env-test file"),
        Err(_) => dotenv::dotenv().expect("Failed to load .env file"),
    };

    let addr = env::var("TOURNAMENT_SERVER_ADDR").expect("TOURNAMENT_SERVER_ADDR");
    let port = env::var("TOURNAMENT_SERVER_PORT").expect("TOURNAMENT_SERVER_PORT");
    let full_addr = format!("{addr}:{port}");
    let api_service = OpenApiService::new(Api, env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
        .server(format!("https://{full_addr}"));
    let ui = api_service.swagger_ui();

    let db_type = env::var("DATABASE_TYPE").expect("DATABASE_TYPE");
    // Box because of dyn
    let db: SharedDb = match db_type.as_str() {
        "SQLITE" => Box::new(Sqlite::default()) as Box<dyn Database + Send + Sync>,
        "INMEMORY" => Box::new(InMemory::default()) as Box<dyn Database + Send + Sync>,
        _ => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "could not parse DATABASE_TYPE",
            ))
        }
    };
    // Arc for clone, RwLock to secure access to db
    let db = Arc::new(RwLock::new(db));
    let cors = Cors::new().allow_method(Method::GET);
    let app = Route::new()
        .nest("/", api_service)
        .nest("/swagger", ui)
        .with(cors)
        .data(db);

    let cert_path = env::var("CERT_PATH").expect("did not find CERT_PATH");
    let key_path = env::var("KEY_PATH").expect("did not find KEY_PATH");
    let cert = std::fs::read_to_string(cert_path).expect("Could not read cert");
    let key = std::fs::read_to_string(key_path).expect("Could not read key");
    let config_stream = RustlsConfig::new().fallback(RustlsCertificate::new().cert(cert).key(key));
    let tls = TcpListener::bind(full_addr).rustls(config_stream);
    //let tls = TcpListener::bind(full_addr);
    poem::Server::new(tls).run(app).await
}
