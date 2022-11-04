use actix_web::{error, get, web, Error, Responder};
use actix_web_lab::respond::Html;

#[get("/status")]
pub async fn status() -> impl Responder {
    "OK"
}

#[get("/")]
pub async fn index(tmpl: web::Data<tera::Tera>) -> Result<impl Responder, Error> {
    let ctx = tera::Context::new();
    let s = tmpl.render("index.html", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template Error"))?;

    Ok(Html(s))
}
