use std::sync::Mutex;
use std::path::PathBuf;

use rocket::response::status::Custom;
use rocket::http::Status;
use rocket_contrib::JSON;
use rocket_contrib::Template;
use rocket::State;
use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;

use common::types::{Url, Key};
use models::{Feed, UserFeed, Entry, UserEntry};
use guards::user::UserGuard;
use schema::{feed, subscription, entry};

type Connection = Mutex<PgConnection>;

#[derive(Serialize)]
struct Context {
    uid: String,
    feed: UserFeed,
    entry: UserEntry
}

#[derive(Serialize)]
struct EmptyContext;

#[get("/<key..>")]
pub fn one(user: UserGuard, conn: State<Connection>, key: PathBuf) -> Template {
    let conn = conn.lock().unwrap();
    use schema::feed::dsl::id as feed_id_field;
    use schema::entry::dsl::key as ekey;
    use schema::entry::dsl::feed_id as entry_feed_id_field;

    let key = Key::from_raw(key.to_str().unwrap().to_owned());
    let entry = match entry::table.filter(ekey.eq(&key)).first::<Entry>(&*conn) {
        Ok(entry) => entry,
        Err(_) => return Template::render("error", &EmptyContext)
    };

    let feed = match feed::table.filter(feed_id_field.eq(entry.feed_id)).first::<Feed>(&*conn) {
        Ok(feed) => feed,
        Err(_) => return Template::render("error", &EmptyContext)
    };

    Template::render("entry", &Context {
        uid: user.uid.to_string(),
        feed: feed.into(),
        entry: entry.into()
    })
}
