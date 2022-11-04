use crate::database::cache::Cache;
use crate::model::{self, APISafe};
use anyhow::Context;
use mongodb::bson::doc;
use mongodb::{Client, Collection};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct DB {
    client: Client,
    pub cache: Arc<Mutex<Cache>>,
    usercolletion: Collection<model::user::DBUser>,
}

impl DB {
    pub async fn new(mongodb_url: &String, redis_url: &String) -> anyhow::Result<Self> {
        let client = Client::with_uri_str(mongodb_url)
            .await
            .context("Could not connect to database")?;

        Ok(DB {
            client: client.clone(),
            cache: Arc::new(Mutex::new(Cache::new(redis_url).await?)),
            usercolletion: client.database("d2o5").collection("users"),
        })
    }

    pub async fn create_user(&self, user: model::user::DBUser) -> anyhow::Result<()> {
        self.usercolletion.insert_one(user, None).await?;

        Ok(())
    }

    pub async fn fetch_user(
        &self,
        username: &String,
    ) -> anyhow::Result<Option<model::user::DBUser>> {
        let user = self
            .usercolletion
            .find_one(doc! {"username": &username}, None)
            .await
            .context("Could not find user")?;

        Ok(user)
    }

    pub async fn user(&mut self, username: &String) -> anyhow::Result<Option<model::user::User>> {
        let user = match self.cache.lock().unwrap().get_user(username).await {
            Ok(user) => Some(user),
            Err(_) => match self.fetch_user(username).await {
                Ok(user) => match user {
                    Some(user) => {
                        self.cache.lock().unwrap().set_user(user.public()).await;
                        Some(user.public())
                    }
                    None => None,
                },
                Err(_) => None,
            },
        };

        Ok(user)
    }
}
