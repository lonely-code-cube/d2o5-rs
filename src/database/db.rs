use crate::database::cache::Cache;
use crate::model::{self, APISafe};
use anyhow::Context;
use mongodb::bson::doc;
use mongodb::{Client, Collection};
use std::sync::Arc;

#[derive(Clone)]
pub struct DB {
    client: Client,
    usercolletion: Collection<model::user::DBUser>,
    usercache: Arc<Cache<String, model::user::User>>,
}

impl DB {
    pub async fn new(url: &String) -> anyhow::Result<Self> {
        let client = Client::with_uri_str(url)
            .await
            .context("Could not connect to database")?;

        Ok(DB {
            client: client.clone(),
            usercolletion: client.database("d2o5").collection("users"),
            usercache: Arc::new(Cache::new()),
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

    pub async fn user(&self, username: &String) -> anyhow::Result<Option<model::user::User>> {
        let user = match self.usercache.get(username) {
            Some(user) => Some(user.to_owned()),
            None => match &self.fetch_user(username).await?.to_owned() {
                Some(user) => Some(user.public()),
                None => None,
            },
        };

        Ok(user)
    }
}
