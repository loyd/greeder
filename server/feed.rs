use rocket::response::status::Custom;
use rocket::http::Status;
use rocket_contrib::JSON;
use rocket::State;
use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use std::sync::Mutex;

use common::types::{Url, Key};
use common::models::{NewFeed, Feed};
use guards::user::UserGuard;
use schema::feed;

type Connection = Mutex<PgConnection>;

#[derive(Deserialize)]
pub struct PostUrl {
    url: String
}

#[post("/add", data = "<url>")]
pub fn add(url: JSON<PostUrl>, user: UserGuard, conn: State<Connection>) -> Custom<()> {
    let url = match Url::parse(&url.0.url) {
        Ok(url) => url,
        Err(err) => return Custom(Status::new(400, "Bad url"), ())
    };

    // TODO(ydz): check feed availability.
    let key = Key::from(url.clone());
    let new_feed = NewFeed {
        key: key,
        url: url
    };
    let conn = conn.lock().unwrap();
    let insertion = diesel::insert(&new_feed)
        .into(feed::table)
        .get_result::<Feed>(&*conn);

    match insertion {
        Ok(_) => Custom(Status::new(200, "Successfully added and subscribed"), ()),
        Err(_) => Custom(Status::new(500, "Addition failed"), ())
    }
}

// #[delete("/remove",)]
// pub fn remove(feed: JSON<Feed>) {

// }
