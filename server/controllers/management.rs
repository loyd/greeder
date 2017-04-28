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
    uid: String
}

#[derive(Serialize)]
struct EmptyContext;

#[get("/")]
pub fn index(user: UserGuard, conn: State<Connection>) -> Template {
    Template::render("management", &Context {
        uid: user.uid.to_string()
    })
}

#[derive(Deserialize)]
pub struct UnsubFeed {
    feed_id: i32
}

#[post("/unsub", data="<feed>")]
pub fn unsub(user: UserGuard, conn: State<Connection>, feed: JSON<UnsubFeed>) -> Custom<()> {
    let conn = conn.lock().unwrap();
    use schema::subscription::dsl::user_id;
    use schema::subscription::dsl::feed_id;

    let sub = subscription::table
        .filter(user_id.eq(user.id))
        .filter(feed_id.eq(feed.0.feed_id))
        .first::<Subscription>(&*conn);
    let sub = match sub {
        Ok(sub) => sub,
        Err(_) => return Custom(Status::new(500, "DB Error"), ())
    };

    match diesel::delete(&sub).execute(&*conn) {
        Ok(_) => Custom(Status::Ok, ()),
        Err(_) => Custom(Status::new(500, "DB Error"), ())
    }
}

#[post("/sub", data="<feed>")]
pub fn sub(user: UserGuard, conn: State<Connection>, feed: JSON<UnsubFeed>) -> Custom<()> {
    let conn = conn.lock().unwrap();
    let sub = Subscription {
        user_id: user.id,
        feed_id: feed.0.feed_id
    };

    match diesel::insert(&sub).into(subscription::table).execute(&*conn) {
        Ok(_) => Custom(Status::Ok, ()),
        Err(_) => Custom(Status::new(500, "DB Error"), ())
    }
}