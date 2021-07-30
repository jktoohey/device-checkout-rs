#![feature(plugin, proc_macro_hygiene, decl_macro)]

//#[macro_use] //this crate has macros, currently unused
pub use failure;
#[macro_use]
pub extern crate log;
pub use stderrlog;
#[macro_use]
pub extern crate clap;
#[macro_use]
pub extern crate rocket;
pub use rocket_contrib;
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
pub use exitfailure;
#[macro_use]
extern crate validator_derive;
use validator;


pub mod database;
pub mod models;
pub mod pool;
pub mod routes;
pub mod schema;
pub mod utils;
