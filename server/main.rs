#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate common;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate rocket;
extern crate rocket_contrib;

use rocket::config::{Config, Environment};
use rocket_contrib::Template;

use common::logger;

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

    let config = Config::build(Environment::Production)
        .port(3000)
        .unwrap();

    rocket::custom(config, false)
        .mount("/", routes![index])
        .launch();
}
