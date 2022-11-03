use actix_web::{App, HttpServer};
use log::{info, warn};
use std::env;

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
    HttpServer::new(|| App::new().service(crate::status::status))
        .bind(("127.0.0.1", port))?
        .run()
        .await
}
