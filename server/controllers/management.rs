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
    feeds: Vec<Feed>
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
pub fn subs(user: UserGuard, conn: State<Connection>, feed: JSON<UnsubFeed>) -> Custom<()> {
    let conn = conn.lock().unwrap();
    use schema::feed::dsl::feed_id;
    use schema::user::dsl::uid as user_id

    let subscription = subscription::table
        .filter(user_id.eq(user.uid))
        .filter(feed_id.eq(feed.id))
        .first::<Subscription>(&*conn);
    let subscription = match subscription {
        Ok(sub) => sub,
        Err(_) => return Custom(Status::new(500, "DB Error"), ())
    };

    match diesel::delete(subscription).execute(&*conn) {
        Ok(_) => Custom(Status::Ok, ()),
        Err(_) => Custom(Status::new(500, "DB Error"), ())
    }
}