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

#[derive(Debug, Queryable, Serialize)]
pub struct UserFeed {
    pub key: Key,
    pub url: Url,
    pub title: Option<String>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub logo: Option<Url>,
    pub copyright: Option<String>
}

impl Into<UserFeed> for Feed {
    fn into(self) -> UserFeed {
        UserFeed {
            key: self.key,
            url: self.url,
            title: self.title,
            description: self.description,
            language: self.language,
            logo: self.logo,
            copyright: self.copyright
        }
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

#[derive(Debug, Queryable, Serialize)]
#[table_name="entry"]
pub struct UserEntry {
    pub key: Key,
    pub url: Option<Url>,
    pub title: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub published: Option<u64>
}

impl Into<UserEntry> for Entry {
    fn into(self) -> UserEntry {
        UserEntry {
            key: self.key,
            url: self.url,
            title: self.title,
            author: self.author,
            description: self.description,
            content: self.content,
            published: self.published.map(|t| t.sec as u64 + (t.nsec / 1000000000) as u64)
        }
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
