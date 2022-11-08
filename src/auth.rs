use crate::{database::db::DB, model, AppData};
use actix_web::{web, Error, FromRequest};
use core::convert::TryFrom;
use futures::Future;
use pasetors::token::UntrustedToken;
use pasetors::{claims::ClaimsValidationRules, local, version4::V4, Local};
use std::pin::Pin;

#[derive(serde::Serialize)]
pub struct AuthUser(pub Option<model::user::User>);

impl AuthUser {
    fn new(user: Option<model::user::User>) -> Self {
        AuthUser(user)
    }
}

impl FromRequest for AuthUser {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let req = req.clone();
        Box::pin(async move {
            let validation_rules = ClaimsValidationRules::new();
            let db = req.app_data::<web::Data<DB>>().unwrap();
            let app_data = req.app_data::<web::Data<AppData>>().unwrap();
            let auth_token = req.cookie("auth");
            match auth_token {
                Some(cookie) => {
                    let untrusted_token =
                        UntrustedToken::<Local, V4>::try_from(cookie.value()).unwrap();
                    let trusted_token = local::decrypt(
                        &app_data.paseto_key,
                        &untrusted_token,
                        &validation_rules,
                        None,
                        None,
                    )
                    .expect("error");

                    let claims = trusted_token.payload_claims().unwrap();
                    let username = claims
                        .get_claim("username")
                        .unwrap()
                        .to_string()
                        .replace('\"', "");

                    let user = db.user(&username).await.unwrap();

                    Ok(AuthUser::new(user))
                }
                None => Ok(AuthUser::new(None)),
            }
        })
    }

    fn extract(req: &actix_web::HttpRequest) -> Self::Future {
        Self::from_request(req, &mut actix_web::dev::Payload::None)
    }
}
