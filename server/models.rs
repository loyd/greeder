use time::Timespec;
use uuid::Uuid;

use common::schema::{feed, user};
use common::types::{Url, Key};

use serde::ser::{Serialize, Serializer, SerializeStruct};

#[derive(Debug, Queryable)]
pub struct Feed {
    pub id: i32,
    pub key: Key,
    pub url: Url,
    pub title: Option<String>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub logo: Option<Url>,
    pub copyright: Option<String>,
    pub interval: Option<i32>,
    pub augmented: Option<Timespec>
}

impl Serialize for Feed {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer {
        let mut state = serializer.serialize_struct("Feed", 5)?;
        let title = match &self.title {
            &Some(ref title) => title.clone(),
            &None => String::new()
        };
        let description = match &self.description {
            &Some(ref description) => description.clone(),
            &None => String::new()
        };
        state.serialize_field("id", &self.id)?;
        state.serialize_field("title", &title)?;
        state.serialize_field("description", &description)?;
        state.serialize_field("logo", &self.logo)?;
        state.serialize_field("url", &self.url)?;
        state.end()
    }
}

#[derive(Debug, Insertable)]
#[table_name="feed"]
pub struct NewFeed {
    pub key: Key,
    pub url: Url
}

#[derive(Debug, Queryable)]
pub struct Entry {
    pub id: i64,
    pub key: Key,
    pub feed_id: i32,
    pub url: Option<Url>,
    pub title: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub published: Option<Timespec>
}

#[derive(Debug, Queryable)]
pub struct User {
    pub id: i32,
    pub uid: Uuid
}

#[derive(Debug, Insertable)]
#[table_name="user"]
pub struct NewUser {
    pub uid: Uuid
}

#[derive(Debug, Queryable)]
pub struct Subscription {
    pub user_id: i32,
    pub feed_id: i32
}