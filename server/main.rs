#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate common;
#[macro_use]
extern crate log;
extern crate time;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_codegen;
extern crate uuid;

use std::sync::Mutex;

use rocket::config::{Config, Environment};

use common::logger;
use common::schema;


mod controllers;
use controllers::{index, feed, entry, management};

mod models;
mod guards;

fn main() {
    logger::init().unwrap();

    // TODO(ydz): what about r2d2?
    let conn = schema::establish_connection().unwrap();

    let config = Config::build(Environment::Production)
        .port(3000)
        .unwrap();
    let index_routes = routes![
        index::index, index::statics,
    ];
    let feed_routes = routes![
        feed::fetch_all, feed::add, feed::one
    ];
    let entry_routes = routes![
        entry::one
    ];
    let management_routes = routes![
        management::index, management::sub,
        management::unsub, management::feed_n_subs
    ];

    rocket::custom(config, false)
        .mount("/", index_routes)
        .mount("/feed", feed_routes)
        .mount("/entry", entry_routes)
        .mount("/management", management_routes)
        .manage(Mutex::new(conn))
        .launch();
}
