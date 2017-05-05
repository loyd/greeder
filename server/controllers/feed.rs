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

use std::net::UdpSocket;
use std::io::Write;
use std::path::{PathBuf, Path};

type Connection = Mutex<PgConnection>;

#[derive(Deserialize)]
pub struct PostUrl {
    url: String
}

fn ipc_send_url(feed_id: i32) {
    let mut socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(sock) => sock,
        Err(e) => {
            error!("Couldn't connect: {:?}", e);
            return;
        }
    };
    let id_str = format!("{}", feed_id);
    socket.send_to(id_str.as_bytes(), "127.0.0.1:3001").unwrap();
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
        return Custom(Status::Ok, ())
    }
    let new_feed = NewFeed {
        key: key,
        url: url.clone()
    };
    return match diesel::insert(&new_feed).into(feed::table).get_result::<Feed>(&*conn) {
        Ok(feed) => {
            ipc_send_url(feed.id);
            let new_sub = Subscription {
                user_id: user.id,
                feed_id: feed.id
            };
            diesel::insert(&new_sub).into(subscription::table).execute(&*conn).unwrap();
            Custom(Status::Ok, ())
        },
        Err(_) => return Custom(Status::new(500, "DB transaction failed"), ())
    }
}

type Feeds = JSON<Vec<UserFeed>>;

#[derive(Serialize)]
struct Context {
    uid: String,
    feed: UserFeed,
    entries: Vec<UserEntry>
}

#[derive(Serialize)]
struct EmptyContext;

#[get("/<key..>")]
pub fn one(user: UserGuard, conn: State<Connection>, key: PathBuf) -> Template {
    let conn = conn.lock().unwrap();
    use schema::feed::dsl::key as fkey;
    use schema::entry::dsl::feed_id as entry_feed_id;

    let key = Key::from_raw(key.to_str().unwrap().to_owned());
    let feed = match feed::table.filter(fkey.eq(&key)).first::<Feed>(&*conn) {
        Ok(feed) => feed,
        Err(_) => return Template::render("error", &EmptyContext)
    };

    let entries = match entry::table.filter(entry_feed_id.eq(feed.id)).load::<Entry>(&*conn) {
        Ok(entries) => entries.into_iter().map(|e| e.into()).collect(),
        Err(_) => return Template::render("error", &EmptyContext)
    };

    Template::render("feed", &Context {
        uid: user.uid.to_string(),
        feed: feed.into(),
        entries: entries
    })
}
