use actix_web::{dev::Service as _, web, App, HttpServer};
use argon2::Argon2;
use futures_util::future::FutureExt;
use log::{info, warn};
use pasetors::{keys::SymmetricKey, version4::V4};
use std::env;
use std::sync::atomic::{AtomicUsize, Ordering};
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

pub struct AppData {
    pub paseto_key: SymmetricKey<V4>,
    pub accesses: AtomicUsize,
    pub active_users: AtomicUsize,
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
        let sk = SymmetricKey::<V4>::from(
            env::var("PASETO_KEY")
                .expect("PASETO_KEY not set")
                .as_bytes(),
        )
        .expect("Invalid PASETO_KEY");
        let app_data = AppData {
            paseto_key: sk,
            accesses: AtomicUsize::new(0),
            active_users: AtomicUsize::new(0),
        };

        App::new()
            .service(status::status)
            .service(status::index)
            .service(routes::auth::register)
            .service(routes::auth::register_post)
            .service(routes::auth::login)
            .service(routes::auth::login_post)
            .app_data(db.clone())
            .app_data(web::Data::new(hasher.clone()))
            .app_data(web::Data::new(tera))
            .app_data(web::Data::new(app_data))
            .wrap_fn(|req, srv| {
                print!("{} ", req.path());
                let app_data = req.app_data::<AppData>();
                if app_data.is_some() {
                    app_data.unwrap().accesses.fetch_add(1, Ordering::Relaxed);
                }
                srv.call(req).map(|res| {
                    println!(
                        "{}",
                        match &res {
                            Ok(a) => format!("{}", a.status()),
                            Err(b) => format!("{:?}", b),
                        }
                    );
                    res
                })
            })
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
