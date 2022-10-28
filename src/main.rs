use dotenv::dotenv;
use failure::ResultExt;
use rocket::{Build, Config, Rocket};
use device_checkout::{create_new_rocket, database, pool, utils};
use device_checkout::routes::{api_routes, html_routes};
use rocket::launch;

#[launch]
fn rocket() -> Rocket<Build> {
    dotenv().ok();
    let mut config = utils::cmdline::parse_cmdline();
    config.module_path = Some(module_path!().into());
    utils::logging::configure_logger(&config);
    database::run_migrations(&config).context("Failed to migrate database").unwrap();

    create_new_rocket(config)

    // Rocket ignition happens automatically by the macro
}