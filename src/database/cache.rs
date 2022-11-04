use redis::{aio::Connection, AsyncCommands, Client};
use std::sync::Mutex;

use crate::model;

pub struct Cache {
    con: Connection,
}

impl Cache {
    pub async fn new(url: &String) -> anyhow::Result<Self> {
        let x = Client::open(url.to_owned())?.get_async_connection().await?;
        Ok(Cache { con: x })
    }

    pub async fn set_user(&mut self, user: model::user::User) {
        self.con
            .hset::<&str, &String, String, ()>(
                "users",
                &user.username,
                serde_json::to_string(&user).unwrap(),
            )
            .await;
    }
}
