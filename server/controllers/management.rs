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
use models::{NewFeed, Feed, UserFeed, Subscription, Entry, UserEntry};
use guards::user::UserGuard;
use schema::{feed, subscription, entry};

type Connection = Mutex<PgConnection>;

#[derive(Serialize)]
struct Context {
    uid: String,
    subs: Vec<UserFeed>
}

#[derive(Serialize)]
struct EmptyContext;

#[derive(Serialize)]
struct ErrorContext {
    msg: String
}

impl ErrorContext {
    fn new(e: &str) -> ErrorContext {
        ErrorContext {
            msg: e.to_string()
        }
    }
}

#[get("/")]
pub fn index(user: UserGuard, conn: State<Connection>) -> Template {
    let conn = conn.lock().unwrap();
    use schema::subscription::dsl::user_id as uid;
    use schema::subscription;
    use schema::feed::dsl::id;

    let subscriptions = subscription::table.filter(uid.eq(user.id)).load::<Subscription>(&*conn);
    let subscriptions = match subscriptions {
        Ok(subs) => subs,
        Err(_) => return Template::render("error", &ErrorContext::new("Cant fetch subscriptions"))
    };

    let mut subs = vec![];
    for sub in subscriptions {
        let part = match feed::table.filter(id.eq(sub.feed_id)).load::<Feed>(&*conn) {
            Ok(data) => data.into_iter().map(|e| e.into()),
            Err(_) => return Template::render("error", &ErrorContext::new("Cant fetch feeds"))
        };
        subs.extend(part);
    }

    Template::render("management", &Context {
        uid: user.uid.to_string(),
        subs: subs
    })
}

#[derive(Serialize)]
pub struct Subs {
    pub subs: Vec<UserFeed>
}

type Feeds = JSON<Subs>;

#[get("/subs")]
pub fn subs(user: UserGuard, conn: State<Connection>) -> Custom<Feeds> {
    let conn = conn.lock().unwrap();
    use schema::subscription::dsl::user_id as uid;
    use schema::subscription;
    use schema::feed::dsl::id;

    let subscriptions = subscription::table.filter(uid.eq(user.id)).load::<Subscription>(&*conn);
    let subscriptions = match subscriptions {
        Ok(subs) => subs,
        Err(_) => return Custom(Status::new(500, "DB Error"), JSON(Subs { subs: vec![] }))
    };

    let mut subs = vec![];
    for sub in subscriptions {
        let part = match feed::table.filter(id.eq(sub.feed_id)).load::<Feed>(&*conn) {
            Ok(data) => data.into_iter().map(|e| e.into()),
            Err(_) => return Custom(Status::new(500, "DB Error"), JSON(Subs { subs: vec![] }))
        };
        subs.extend(part);
    }

    Custom(Status::Ok, JSON(Subs { subs }))
}

#[derive(Deserialize)]
pub struct UnsubFeed {
    key: String
}

#[post("/unsub", data="<feed>")]
pub fn unsub(user: UserGuard, conn: State<Connection>, feed: JSON<UnsubFeed>) -> Custom<()> {
    let conn = conn.lock().unwrap();
    use schema::feed::dsl::key as fkey;
    use schema::subscription::dsl::user_id;
    use schema::subscription::dsl::feed_id;

    let key = Key::from_raw(feed.key.clone());
    let unsub_feed = match feed::table.filter(fkey.eq(&key)).first::<Feed>(&*conn) {
        Ok(feed) => feed,
        Err(not_found) => return Custom(Status::new(401, "No such feed"), ())
    };

    let sub = subscription::table
        .filter(user_id.eq(user.id))
        .filter(feed_id.eq(unsub_feed.id))
        .first::<Subscription>(&*conn);
    let sub = match sub {
        Ok(sub) => sub,
        Err(not_found) => return Custom(Status::new(401, "No sub to unsub"), ())
    };

    match diesel::delete(&sub).execute(&*conn) {
        Ok(_) => Custom(Status::Ok, ()),
        Err(_) => Custom(Status::new(500, "DB Error"), ())
    }
}