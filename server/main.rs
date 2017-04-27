#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate common;
#[macro_use]
extern crate log;
extern crate time;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate uuid;
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
use controllers::{pages, feed};

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
        pages::index, pages::statics,
    ];
    let feed_routes = routes![
        feed::fetch_all, feed::add
    ];

    rocket::custom(config, false)
        .mount("/", index_routes)
        .mount("/feed", feed_routes)
        .manage(Mutex::new(conn))
        .launch();
}
