use crate::model;
use redis::{aio::Connection, AsyncCommands, Client};

pub struct Cache {
    con: Connection,
}

impl Cache {
    pub async fn new(url: &String) -> anyhow::Result<Self> {
        let x = Client::open(url.to_owned())?.get_async_connection().await?;
        Ok(Cache { con: x })
    }

    pub async fn set_user(&mut self, user: model::user::User) {
        let _ = self
            .con
            .hset::<&str, &String, String, ()>(
                "d2o5.users",
                &user.username,
                serde_json::to_string(&user).unwrap(),
            )
            .await;
    }

    pub async fn get_user(&mut self, username: &String) -> anyhow::Result<model::user::User> {
        let user_ser = self
            .con
            .hget::<&str, &String, String>("d2o5.users", username)
            .await?;
        Ok(serde_json::from_str::<model::user::User>(&user_ser)?)
    }

    pub async fn remove_user(&mut self, username: &String) -> anyhow::Result<()> {
        Ok(self.con.hdel("d2o5.users", username).await?)
    }
}
