use crate::{
    auth::AuthUser,
    database::db::DB,
    model::{self, APISafe},
    AppData, Hasher,
};
use actix_web::{
    cookie, cookie::Cookie, error, get, http::header::LOCATION, post, web, Error, HttpResponse,
    Responder,
};
use actix_web_lab::respond::Html;
use argon2::{PasswordHash, PasswordVerifier};
use chrono::{Duration, Utc};
use log::error;
use pasetors::{claims::Claims, local};
use serde::Deserialize;
use tera::{Context, Tera};

#[derive(Deserialize)]
pub struct LoginUser {
    username: String,
    password: String,
}

#[get("/register")]
pub async fn register(tmpl: web::Data<Tera>) -> Result<impl Responder, Error> {
    let ctx = Context::new();
    let s = tmpl
        .render("register.html", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Templating Error"))?;

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

            Ok(HttpResponse::SeeOther()
                .insert_header((LOCATION, "/login"))
                .finish())
        }
    }
}

#[get("/login")]
pub async fn login(tmpl: web::Data<Tera>) -> Result<impl Responder, Error> {
    let ctx = Context::new();
    let s = tmpl
        .render("login.html", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Templating Error"))?;

    Ok(Html(s))
}

#[post("/login")]
pub async fn login_post(
    user: web::Form<LoginUser>,
    db: web::Data<DB>,
    hasher: web::Data<Hasher<'_>>,
    app_data: web::Data<AppData>,
) -> Result<impl Responder, Error> {
    if user.username.len() > 20
        || user.username.len() < 2
        || user.password.len() > 20
        || user.password.len() < 5
    {
        return Ok(HttpResponse::BadRequest().finish());
    }

    let iu = db
        .fetch_user(&user.username)
        .await
        .map_err(|_| error::ErrorInternalServerError("Database error"))?;

    match iu {
        Some(dbuser) => {
            let expected_hash = PasswordHash::new(&dbuser.password_hash)
                .map_err(|_| error::ErrorInternalServerError("A unexpected error occured"))?;
            hasher
                .get_hasher()
                .verify_password(user.password.as_bytes(), &expected_hash)
                .map_err(|_| error::ErrorForbidden("Useranme and password do not match"))?;

            let mut claims = Claims::new().map_err(|_| {
                error!("Cound not create paseto claims");
                error::ErrorInternalServerError("An unexpected error occured")
            })?;
            claims
                .expiration(&format!("{:?}", Utc::now() + Duration::days(7)))
                .map_err(|e| {
                    error!("{}", e);
                    error::ErrorInternalServerError("An unexpected error occured")
                })?;
            claims
                .add_additional("username", user.username.to_owned())
                .map_err(|e| {
                    error!("{}", e);
                    error::ErrorInternalServerError("An unexpected error occured")
                })?;
            let token = local::encrypt(&app_data.paseto_key, &claims, None, None).map_err(|e| {
                error!("{}", e);
                error::ErrorInternalServerError("An unexpected error occured")
            })?;
            let mut c = Cookie::new("auth", token);
            c.set_http_only(Some(true));
            c.set_expires(
                cookie::time::OffsetDateTime::now_utc() + cookie::time::Duration::days(7),
            );
            c.set_secure(true);

            db.cache
                .lock()
                .map_err(|e| {
                    error!("{}", e);
                    error::ErrorInternalServerError("An unexpected error occured")
                })?
                .set_user(dbuser.public())
                .await;

            Ok(HttpResponse::SeeOther()
                .cookie(c)
                .insert_header((LOCATION, "/"))
                .finish())
        }
        None => Ok(HttpResponse::NotFound().body("No such user exists")),
    }
}

#[get("/logout")]
pub async fn logout(user: AuthUser, db: web::Data<DB>) -> impl Responder {
    match user.0 {
        Some(user) => {
            let _ = db.cache.lock().unwrap().remove_user(&user.username).await;
            let mut res = HttpResponse::Ok().insert_header((LOCATION, "/")).finish();
            let _ = res.add_removal_cookie(&Cookie::named("auth"));
            res
        }
        None => HttpResponse::Unauthorized().finish(),
    }
}

#[get("/me")]
pub async fn me(user: AuthUser) -> impl Responder {
    HttpResponse::Ok().json(user)
}
