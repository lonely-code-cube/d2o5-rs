use crate::{database::db::DB, model, Hasher};
use actix_web::{error, get, post, web, Error, HttpResponse, Responder};
use actix_web_lab::respond::Html;
use tera::{Context, Tera};

#[get("/register")]
pub async fn register(tmpl: web::Data<Tera>) -> Result<impl Responder, Error> {
    let ctx = Context::new();
    let s = tmpl
        .render("register.html", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template Error"))?;

    Ok(Html(s))
}

#[post("/register")]
pub async fn register_post(
    user: web::Form<model::user::CreateUser>,
    db: web::Data<DB>,
    hasher: web::Data<Hasher<'_>>,
) -> Result<impl Responder, Error> {
    let iu = db
        .fetch_user(&user.username)
        .await
        .map_err(|_| error::ErrorInternalServerError("Database error"))?;

    match iu {
        Some(_) => Ok(HttpResponse::Conflict().body("User already exists")),
        None => {
            if user.username.len() > 20
                || user.username.len() < 2
                || user.password.len() > 20
                || user.password.len() < 5
            {
                return Ok(HttpResponse::BadRequest().finish());
            }

            let dbuser = user
                .dbuser(hasher.get_hasher())
                .map_err(|_| error::ErrorInternalServerError("Could not process user"))?;

            db.create_user(dbuser)
                .await
                .map_err(|_| error::ErrorInternalServerError("COuld not create user"))?;

            Ok(HttpResponse::PermanentRedirect()
                .insert_header(("LOCATION", "/"))
                .finish())
        }
    }
}
