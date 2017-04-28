use time::{Timespec, Tm, strftime, self};
use uuid::Uuid;

use common::schema::{feed, user, subscription};
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

impl Serialize for Entry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer {
        let mut state = serializer.serialize_struct("Entry", 8)?;
        let title = match &self.title {
            &Some(ref title) => title.clone(),
            &None => String::new()
        };
        let description = match &self.description {
            &Some(ref description) => description.clone(),
            &None => String::new()
        };
        let author = match &self.author {
            &Some(ref author) => author.clone(),
            &None => String::new()
        };
        let content = match &self.content {
            &Some(ref content) => content.clone(),
            &None => String::new()
        };
        let published = match &self.published {
            &Some(ref ts) => {
                let tm = time::at(ts.clone());
                strftime("%d.%m.%Y %H:%M", &tm).unwrap()
                // ts.sec + (ts.nsec / 1000000000) as i64
            },
            &None => String::new()
        };
        state.serialize_field("id", &self.id)?;
        state.serialize_field("feed_id", &self.feed_id)?;
        state.serialize_field("url", &self.url)?;
        state.serialize_field("title", &title)?;
        state.serialize_field("description", &description)?;
        state.serialize_field("author", &author)?;
        state.serialize_field("content", &content)?;
        state.serialize_field("published", &published)?;
        state.end()
    }
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

#[derive(Debug, Queryable, Identifiable, Insertable)]
#[table_name="subscription"]
#[primary_key(user_id, feed_id)]
pub struct Subscription {
    pub user_id: i32,
    pub feed_id: i32
}