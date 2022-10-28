//#[macro_use] //this crate has macros, currently unused
pub use failure;
#[macro_use]
pub extern crate log;
pub use stderrlog;
#[macro_use]
pub extern crate clap;
#[macro_use]
pub extern crate rocket;
pub use rocket_dyn_templates;
#[macro_use]
pub extern crate serde_derive;
pub use serde;
pub use serde_json;
#[macro_use]
pub extern crate diesel;
#[macro_use]
pub extern crate diesel_migrations;
#[macro_use]
pub extern crate diesel_derive_enum;
pub use chrono;
use dotenv::dotenv;
pub use exitfailure;
use failure::ResultExt;
use rocket::{Build, Config, Request, response, Rocket};
use rocket::config::SecretKey;
use rocket::response::Responder;

#[macro_use]
extern crate validator_derive;
use validator;
use crate::routes::{api_routes, html_routes};

pub mod database;
pub mod models;
pub mod pool;
pub mod routes;
pub mod schema;
pub mod utils;

pub fn create_new_rocket(config: utils::types::Settings) -> Rocket<Build> {
    // Create the rocket config
    let rocket_config = Config {
        port: config.port,
        secret_key: SecretKey::generate().unwrap(),
        ..Config::release_default()
    };

    rocket::custom(&rocket_config)
        .manage(pool::init_pool(&config))
        .manage(config)
        .attach(rocket_dyn_templates::Template::fairing())
        .mount("/", html_routes())
        .mount("/api/", api_routes())
}