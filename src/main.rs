use actix_web::{web, App, HttpServer};
use log::{info, warn};
use std::env;

mod database;
mod model;
mod status;

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
        database::db::DB::new(&env::var("MONGODB_URL").expect("MONGODB_URL must be set"))
            .await
            .unwrap(),
    );

    HttpServer::new(move || {
        App::new()
            .service(crate::status::status)
            .app_data(db.clone())
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
