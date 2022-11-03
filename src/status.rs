use actix_web::{get, Responder};

#[get("/status")]
pub async fn status() -> impl Responder {
    "OK"
}
