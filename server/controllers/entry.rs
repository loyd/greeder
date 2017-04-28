use rocket::response::status::Custom;
use rocket::http::Status;
use rocket_contrib::JSON;
use rocket_contrib::Template;
use rocket::State;
use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use std::sync::Mutex;

use common::types::{Url, Key};
use models::{NewFeed, Feed, Subscription, Entry};
use guards::user::UserGuard;
use schema::{feed, subscription, entry};

type Connection = Mutex<PgConnection>;

#[derive(Serialize)]
struct Context {
    uid: String,
    feed: Feed,
    entry: Entry
}

#[derive(Serialize)]
struct EmptyContext;

#[get("/<entry_id>")]
pub fn one(user: UserGuard, conn: State<Connection>, entry_id: i64) -> Template {
    let conn = conn.lock().unwrap();
    use schema::feed::dsl::id as feed_id_field;
    use schema::entry::dsl::id as entry_id_field;
    use schema::entry::dsl::feed_id as entry_feed_id_field;

    let entry = match entry::table.filter(entry_id_field.eq(entry_id)).first::<Entry>(&*conn) {
        Ok(entry) => entry,
        Err(_) => return Template::render("error", &EmptyContext)
    };

    let feed = match feed::table.filter(feed_id_field.eq(entry.feed_id)).first::<Feed>(&*conn) {
        Ok(feed) => feed,
        Err(_) => return Template::render("error", &EmptyContext)
    };

    Template::render("entry", &Context {
        uid: user.uid.to_string(),
        feed: feed,
        entry: entry
    })
}
