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

use std::net::UdpSocket;
use std::io::Write;

type Connection = Mutex<PgConnection>;

#[derive(Deserialize)]
pub struct PostUrl {
    url: String
}

fn ipc_send_url(url: Url) {
    let mut socket = match UdpSocket::bind("127.0.0.1:3001") {
        Ok(sock) => sock,
        Err(e) => {
            error!("Couldn't connect: {:?}", e);
            return;
        }
    };
    let bytes = url.to_string();
    socket.send(bytes.as_bytes());
}

#[post("/add", data = "<url>")]
pub fn add(url: JSON<PostUrl>, user: UserGuard, conn: State<Connection>) -> Custom<()> {
    let url = match Url::parse(&url.0.url) {
        Ok(url) => url,
        Err(err) => return Custom(Status::new(400, "Bad url"), ())
    };
    let key = Key::from(url.clone());
    use schema::feed::dsl::key as key_fld;
    let conn = conn.lock().unwrap();
    let feed_exists = match feed::table.filter(key_fld.eq(&key)).load::<Feed>(&*conn) {
        Ok(feeds) => feeds.len() != 0,
        Err(_) => return Custom(Status::new(500, "DB transaction failed"), ())
    };

    if feed_exists {
        Custom(Status::Ok, ())
    } else {
        let new_feed = NewFeed {
            key: key,
            url: url.clone()
        };
        match diesel::insert(&new_feed).into(feed::table).execute(&*conn) {
            Ok(feed) => {
                ipc_send_url(url);
                Custom(Status::Ok, ())
            },
            Err(_) => return Custom(Status::new(500, "DB transaction failed"), ())
        }
    }
}

type Feeds = JSON<Vec<Feed>>;

#[get("/")]
pub fn fetch_all(user: UserGuard, conn: State<Connection>) -> Custom<Feeds> {
    let conn = conn.lock().unwrap();
    use schema::subscription::dsl::user_id;
    use schema::subscription;

    let subscriptions = subscription::table
        .filter(user_id.eq(user.0.id))
        .load::<Subscription>(&*conn);
    let subscriptions = match subscriptions {
        Ok(subs) => subs,
        Err(_) => return Custom(Status::new(500, "DB Error"), JSON(vec![]))
    };

    let mut feeds = vec![];
    use schema::feed::dsl::id;
    for sub in subscriptions {
        let feed_id = sub.feed_id;
        let part = feed::table
            .filter(id.eq(feed_id))
            .load::<Feed>(&*conn);
        let part = match part {
            Ok(data) => data,
            Err(_) => return Custom(Status::new(500, "DB Error"), JSON(vec![]))
        };
        feeds.extend(part);
    }

    Custom(Status::Ok, JSON(feeds))
}

#[derive(Serialize)]
struct Context {
    uid: String,
    feed: Feed,
    entries: Vec<Entry>
}

#[derive(Serialize)]
struct EmptyContext;

#[get("/<feed_id>")]
pub fn one(user: UserGuard, conn: State<Connection>, feed_id: i32) -> Template {
    let conn = conn.lock().unwrap();
    use schema::feed::dsl::id;
    use schema::entry::dsl::feed_id as entry_feed_id;
    let feed = match feed::table.filter(id.eq(feed_id)).first::<Feed>(&*conn) {
        Ok(feed) => feed,
        Err(_) => return Template::render("error", &EmptyContext)
    };

    let entries = match entry::table.filter(entry_feed_id.eq(feed_id)).load::<Entry>(&*conn) {
        Ok(entries) => entries,
        Err(_) => return Template::render("error", &EmptyContext)
    };

    Template::render("feed", &Context {
        uid: user.uid.to_string(),
        feed: feed,
        entries: entries
    })
}
// #[delete("/remove",)]
// pub fn remove(feed: JSON<Feed>) {

// }
