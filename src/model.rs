pub trait APISafe<T> {
    fn public(&self) -> T;
}

pub mod user {
    use super::APISafe;
    use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
    use chrono::{DateTime, Utc};
    use mongodb::bson::oid::ObjectId;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct DBUser {
        #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
        pub id: Option<ObjectId>,
        #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
        pub created_at: DateTime<Utc>,
        pub username: String,
        pub display_name: String,
        pub avatar_url: Option<String>,
        pub password_hash: String,
        pub salt: String,
    }

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct User {
        pub id: String,
        #[serde(
            rename = "createdAt",
            with = "bson::serde_helpers::chrono_datetime_as_bson_datetime"
        )]
        pub created_at: DateTime<Utc>,
        pub username: String,
        #[serde(rename = "displayName")]
        pub display_name: String,
        #[serde(rename = "avatarUrl")]
        pub avatar_url: Option<String>,
    }

    #[derive(Debug, Deserialize, Clone)]
    pub struct CreateUser {
        pub username: String,
        pub password: String,
        #[serde(rename = "displayName")]
        pub display_name: Option<String>,
        #[serde(rename = "avatarUrl")]
        pub avatar_url: Option<String>,
    }

    impl CreateUser {
        pub fn dbuser(&self, hasher: &Argon2) -> Result<DBUser, anyhow::Error> {
            let salt = SaltString::generate(&mut rand::thread_rng());
            let password_hash = hasher
                .hash_password(self.password.as_bytes(), &salt)
                .unwrap()
                .to_string();

            Ok(DBUser {
                id: None,
                created_at: Utc::now(),
                username: self.username.to_owned(),
                display_name: match &self.display_name {
                    Some(dn) => {
                        if dn.is_empty() || dn.len() > 20 {
                            self.username.to_owned()
                        } else {
                            dn.to_owned()
                        }
                    }
                    None => self.username.to_owned(),
                },
                avatar_url: match &self.avatar_url {
                    Some(url) => {
                        if url.is_empty() {
                            None
                        } else {
                            Some(url.to_owned())
                        }
                    }
                    None => None,
                },
                password_hash: password_hash,
                salt: salt.to_string(),
            })
        }
    }

    impl APISafe<User> for DBUser {
        fn public(&self) -> User {
            User {
                id: self.id.unwrap().to_string(),
                username: self.username.to_owned(),
                created_at: self.created_at,
                display_name: self.display_name.to_owned(),
                avatar_url: self.avatar_url.to_owned(),
            }
        }
    }
}
