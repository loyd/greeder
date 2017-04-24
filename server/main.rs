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
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_codegen;

use std::sync::Mutex;

use rocket::config::{Config, Environment};
use rocket_contrib::Template;

use common::logger;
use common::schema;

mod models;

#[get("/")]
pub fn index() -> Template {
    #[derive(Serialize)]
    struct Context {
        name: String
    }

    Template::render("index", &Context {
        name: "world".to_owned()
    })
}

fn main() {
    logger::init().unwrap();

    // TODO(ydz): what about r2d2?
    let conn = schema::establish_connection().unwrap();

    let config = Config::build(Environment::Production)
        .port(3000)
        .unwrap();

    rocket::custom(config, false)
        .mount("/", routes![index])
        .manage(Mutex::new(conn))
        .launch();
}
