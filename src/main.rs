use actix_web::{middleware::Logger, web, App, HttpServer};
use argon2::Argon2;
use log::{info, warn};
use std::env;
use tera::Tera;

mod database;
mod model;
mod routes;
mod status;

#[derive(Clone)]
pub struct Hasher<'a>(Argon2<'a>);
impl Hasher<'_> {
    pub fn new() -> Self {
        Hasher(Argon2::default())
    }
    pub fn get_hasher(&self) -> &Argon2 {
        &self.0
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::builder()
        .parse_env("LOG_LEVEL")
        .format_timestamp(None)
        .init();

    let port = match env::var("PORT") {
        Ok(port) => {
            info!("Starting Server at {}:{}", "127.0.0.1", port);
            port.parse::<u16>().expect("PORT must be a u16 value")
        }
        Err(_) => {
            warn!("PORT env variable is not set. USing 8070 as default");
            info!("Starting Server at {}:{}", "127.0.0.1", 8070);
            8070
        }
    };

    info!("Connectiong to database");
    let db = web::Data::new(
        database::db::DB::new(
            &env::var("MONGODB_URL").expect("MONGODB_URL must be set"),
            &env::var("REDIS_URL").expect("REDIS_URL must be set"),
        )
        .await
        .unwrap(),
    );

    let hasher = Hasher::new();

    HttpServer::new(move || {
        let tera = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();

        App::new()
            .service(status::status)
            .service(status::index)
            .service(routes::auth::register)
            .service(routes::auth::register_post)
            .app_data(db.clone())
            .app_data(web::Data::new(hasher.clone()))
            .app_data(web::Data::new(tera))
            .wrap(Logger::default())
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
