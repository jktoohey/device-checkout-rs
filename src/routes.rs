#![cfg_attr(feature = "cargo-clippy", allow(print_literal))]

use chrono;
use chrono::Offset;
use crate::database;
use failure;
use crate::models;
use crate::pool;
use rocket_dyn_templates;
use std;
use failure::{Error, Fail};
use rocket::form::Form;
use rocket::serde::json;
use crate::utils;
use validator;
use validator::Validate;
use crate::models::{CustomOwner, Device, Pool};

pub fn html_routes() -> Vec<rocket::Route> {
    routes![
        self::index,
        self::get_devices,
        self::post_devices,
        self::get_edit_devices,
        self::post_edit_devices,
        self::post_add_devices,
        self::post_delete_devices,
        self::get_edit_pools,
        self::post_edit_pools,
        self::post_add_pools,
        self::post_delete_pools,
        self::get_edit_custom_owners,
        self::post_edit_custom_owners,
        self::post_add_custom_owners,
        self::post_delete_custom_owners,
    ]
}

pub fn api_routes() -> Vec<rocket::Route> {
    routes![
        self::api_get_device,
        self::api_get_devices,
        self::api_get_pools,
        self::api_get_custom_owner,
        self::api_get_custom_owners,
        self::api_post_reservations,
        self::api_delete_reservation,
    ]
}

#[get("/")]
pub fn index() -> rocket::response::Redirect {
    trace!("index()");
    rocket::response::Redirect::to("/devices")
}

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
#[get("/devices/<name>")]
pub fn api_get_device(
    config: &rocket::State<utils::types::Settings>,
    database: pool::DbConn,
    name: String,
) -> Result<json::Json<models::Device>, rocket::response::status::Custom<String>> {
    trace!("api_get_device()");
    database::get_device(&*config, &*database, &name)
        .map_err(|_| {
            rocket::response::status::Custom(
                rocket::http::Status::InternalServerError,
                "500 Internal Server Error".to_string(),
            )
        })
        .and_then(|devices| {
            devices.ok_or_else(|| {
                rocket::response::status::Custom(
                    rocket::http::Status::NotFound,
                    "404 Not Found".to_string(),
                )
            })
        })
        .map(json::Json)
}

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
#[get("/devices")]
pub fn api_get_devices(
    config: &rocket::State<utils::types::Settings>,
    database: pool::DbConn,
) -> Result<json::Json<Vec<Device>>, String> {
    trace!("api_get_devices()");
    return match database::get_devices(&*config, &*database) {
        Ok(devices) => {
            Ok(json::Json(devices))
        }
        Err(e) => {
            Err(e.to_string())
        }
    };
}

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
#[post("/reservations", format = "application/json", data = "<reservation>")]
pub fn api_post_reservations(
    config: &rocket::State<utils::types::Settings>,
    database: pool::DbConn,
    reservation: json::Json<models::ReservationRequest>,
) -> Result<json::Json<models::Reservation>, rocket::http::Status> {
    trace!("api_post_reservations");
    // Find an available device from the pool specified
    let available_device =
        database::get_available_device_from_pool(&*config, &*database, &reservation.device.pool_id)
            .map_err(|_| rocket::http::Status::InternalServerError)
            .and_then(|devices| devices.ok_or_else(|| rocket::http::Status::NotFound))?;
    // Set the device as reserved
    let update_reserved_device = models::DeviceUpdate {
        id: available_device.id,
        device_owner: reservation.device_owner.clone(),
        comments: reservation.comments.clone(),
        reservation_status: models::ReservationStatus::Reserved,
    };
    let updated_device = match database::update_device(
        &*config,
        &*database,
        &update_reserved_device,
        models::ReservationStatus::Available,
    ) {
        Ok(0) | Err(_) => Err(rocket::http::Status::InternalServerError),
        _ => database::get_device(&*config, &*database, &available_device.device_name)
            .map_err(|_| rocket::http::Status::InternalServerError)
            .and_then(|devices| devices.ok_or_else(|| rocket::http::Status::NotFound)),
    }?;
    // Return a reservation response with the reserved device
    let reservation_response = models::Reservation {
        id: updated_device.id.clone(),
        device: updated_device,
        device_owner: reservation.device_owner.clone().unwrap(),
        comments: reservation.comments.clone(),
    };
    Ok(json::Json(reservation_response))
}

#[delete("/reservations/<id>")]
pub fn api_delete_reservation(
    config: &rocket::State<utils::types::Settings>,
    database: pool::DbConn,
    id: i32,
) -> rocket::http::Status {
    trace!("api_delete_reservation()");
    /* For now the id of a reservation is the id of the device. */
    if let Ok(None) = database::get_device_by_id(&*config, &*database, id) {
        return rocket::http::Status::NotFound;
    }

    let device_update = models::DeviceUpdate {
        id: id,
        reservation_status: models::ReservationStatus::Available,
        comments: None,
        device_owner: None,
    };
    let update_result = database::update_device(
        &*config,
        &*database,
        &device_update,
        models::ReservationStatus::Reserved,
    );

    match update_result {
        Ok(0) => rocket::http::Status::BadRequest,
        Err(_) => rocket::http::Status::InternalServerError,
        _ => rocket::http::Status::NoContent,
    }
}

#[derive(Serialize)]
struct PerDeviceContext {
    device: models::Device,
    is_reserved: bool,
    updated_at_local: String,
}

#[derive(Serialize, Default)]
struct DevicesContext<'a> {
    devices: Vec<PerDeviceContext>,
    pools: Vec<models::Pool>,
    current_pool: Option<models::Pool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_message: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    success_message: Option<&'a str>,
}

fn format_device(device: models::Device) -> PerDeviceContext {
    let is_reserved = device.reservation_status == models::ReservationStatus::Reserved;
    let updated_at_local = chrono::DateTime::<chrono::Local>::from_utc(
        device.updated_at,
        chrono::Local::now().offset().fix(),
    );
    trace!("format_device");

    let updated_at_local = format!("{}", updated_at_local.format("%F %r"));
    PerDeviceContext {
        device,
        is_reserved,
        updated_at_local,
    }
}

fn gen_device_context<'a>(
    config: &utils::types::Settings,
    database: &database::DbConn,
    status_message: &'a Option<rocket::request::FlashMessage<'_>>,
    requested_pool_id: Option<i32>,
) -> Result<DevicesContext<'a>, failure::Error> {
    trace!("gen_device_context");

    let mut success_message = None;
    let mut error_message = None;

    if let Some(ref status_message) = *status_message {
        if status_message.kind() == "success" {
            success_message = Some(status_message.message());
        } else {
            error_message = Some(status_message.message());
        }
    }

    let pools: Vec<_> = database::get_pools(config, database)?;
    let unformatted_devices: Vec<_>;
    let current_pool: Option<models::Pool>;
    if let Some(pool_id) = requested_pool_id {
        unformatted_devices = database::get_devices_in_pool(config, database, pool_id)?;
        current_pool = Some(database::get_pool_by_id(config, database, pool_id)?);
    } else {
        unformatted_devices = database::get_devices(config, database)?;
        current_pool = None;
    };
    let devices = unformatted_devices.into_iter()
        .map(format_device)
        .collect();

    Ok(DevicesContext {
        devices,
        pools,
        current_pool,
        error_message,
        success_message,
    })
}

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
#[get("/devices?<pool_id>")]
pub fn get_devices(
    config: &rocket::State<utils::types::Settings>,
    database: pool::DbConn,
    status_message: Option<rocket::request::FlashMessage<'_>>,
    pool_id: Option<i32>,
) -> Result<rocket_dyn_templates::Template, String> {
    trace!("get_devices()");

    return match gen_device_context(&*config, &*database, &status_message, pool_id) {
        Ok(context) => {
            Ok(rocket_dyn_templates::Template::render(
            "devices", &context,
            ))
        }
        Err(e) => {
            Err(e.to_string())
        }
    };
}

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
#[get("/editDevices")]
pub fn get_edit_devices(
    config: &rocket::State<utils::types::Settings>,
    database: pool::DbConn,
    status_message: Option<rocket::request::FlashMessage<'_>>,
) -> Result<rocket_dyn_templates::Template, String> {
    trace!("get_edit_devices()");

    match gen_device_context(&*config, &*database, &status_message, None) {
        Ok(context) => {
            Ok(rocket_dyn_templates::Template::render(
                "edit_devices",
                &context,
            ))
        }
        Err(e) => {
            Err(e.to_string())
        }
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
#[post("/addDevices", data = "<device_add>")]
pub fn post_add_devices(
    config: &rocket::State<utils::types::Settings>,
    database: pool::DbConn,
    device_add: Form<models::DeviceInsert>,
) -> rocket::response::Flash<rocket::response::Redirect> {
    trace!("post_add_devices()");

    let device = device_add.into_inner();
    if let Err(errors) = device.validate() {
        let errors = errors.field_errors();
        let msg = match find_first_validation_message(&errors) {
            Some(m) => m,
            None => "Failed to parse form data",
        };
        return rocket::response::Flash::error(
            rocket::response::Redirect::to("/editDevices"),
            msg,
        );
    }
    let add_result = database::insert_device(&*config, &*database, &device);

    match add_result {
        Ok(0) | Err(_) => rocket::response::Flash::error(
            rocket::response::Redirect::to("/editDevices"),
            "Failed to add device",
        ),
        _ => rocket::response::Flash::success(
            rocket::response::Redirect::to("/editDevices"),
            "Successfully added device",
        ),
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
#[post("/deleteDevices", data = "<device_edit>")]
pub fn post_delete_devices(
    config: &rocket::State<utils::types::Settings>,
    database: pool::DbConn,
    device_edit: Form<models::DeviceDelete>,
) -> rocket::response::Flash<rocket::response::Redirect> {
    trace!("post_delete_devices()");

    let device = device_edit.into_inner();
    let update_result = database::delete_device(&*config, &*database, &device);

    match update_result {
        Ok(0) | Err(_) => rocket::response::Flash::error(
            rocket::response::Redirect::to("/editDevices"),
            "Failed to delete device",
        ),
        _ => rocket::response::Flash::success(
            rocket::response::Redirect::to("/editDevices"),
            "Successfully deleted device",
        ),
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
#[post("/editDevices", data = "<device_edit>")]
pub fn post_edit_devices(
    config: &rocket::State<utils::types::Settings>,
    database: pool::DbConn,
    device_edit: Form<models::DeviceEdit>,
) -> rocket::response::Flash<rocket::response::Redirect> {
    trace!("post_edit_devices()");

    let device = device_edit.into_inner();
    if let Err(errors) = device.validate() {
        let errors = errors.field_errors();
        let msg = match find_first_validation_message(&errors) {
            Some(m) => m,
            None => "Failed to parse form data",
        };
        return rocket::response::Flash::error(
            rocket::response::Redirect::to("/editDevices"),
            msg,
        );
    }
    let update_result = database::edit_device(&*config, &*database, &device);

    match update_result {
        Ok(0) | Err(_) => rocket::response::Flash::error(
            rocket::response::Redirect::to("/editDevices"),
            "Failed to update device",
        ),
        _ => rocket::response::Flash::success(
            rocket::response::Redirect::to("/editDevices"),
            "Successfully updated device",
        ),
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
#[post("/devices", data = "<device_update>")]
pub fn post_devices(
    config: &rocket::State<utils::types::Settings>,
    database: pool::DbConn,
    device_update: Form<models::DeviceUpdate>
) -> rocket::response::Flash<rocket::response::Redirect> {
    trace!("post_devices()");

    let mut device = device_update.into_inner();

    //save the old reservation status around for the sql query
    let current_reservation_status = device.reservation_status;

    //toggle the reservation status
    device.reservation_status = !device.reservation_status;

    //blank out the owner and comments if we're returning it
    if device.reservation_status == models::ReservationStatus::Available {
        device.device_owner = None;
        device.comments = None;
    }

    if let Err(errors) = device.validate() {
        let errors = errors.field_errors();
        let msg = match find_first_validation_message(&errors) {
            Some(m) => m,
            None => "Failed to parse form data",
        };
        return rocket::response::Flash::error(rocket::response::Redirect::to("/devices"), msg);
    }

    let update_result = database::update_device(&*config, &*database, &device, current_reservation_status);

    match update_result {
        Ok(0) | Err(_) => rocket::response::Flash::error(
            rocket::response::Redirect::to("/devices"),
            "Failed to update device",
        ),
        _ => rocket::response::Flash::success(
            rocket::response::Redirect::to("/devices"),
            "Successfully updated device",
        ),
    }
}

// pools
#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
#[get("/pools")]
pub fn api_get_pools(
    config: &rocket::State<utils::types::Settings>,
    database: pool::DbConn,
) -> Result<json::Json<Vec<models::Pool>>, String> {
    trace!("api_get_pools()");
    return match database::get_pools(&*config, &*database) {
        Ok(pools) => {
            Ok(json::Json(pools))
        }
        Err(e) => {
            Err(e.to_string())
        }
    };
}

#[derive(Serialize)]
struct PerPoolContext {
    pool: models::Pool,
    updated_at_local: String,
}

#[derive(Serialize, Default)]
struct PoolContext<'a> {
    pools: Vec<PerPoolContext>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_message: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    success_message: Option<&'a str>,
}

fn format_pool(pool: models::Pool) -> PerPoolContext {
    let updated_at_local = chrono::DateTime::<chrono::Local>::from_utc(
        pool.updated_at,
        chrono::Local::now().offset().fix(),
    );
    trace!("format_pool");

    let updated_at_local = format!("{}", updated_at_local.format("%F %r"));
    PerPoolContext {
        pool,
        updated_at_local,
    }
}

fn gen_pool_context<'a>(
    config: &utils::types::Settings,
    database: &database::DbConn,
    status_message: &'a Option<rocket::request::FlashMessage<'_>>,
) -> Result<PoolContext<'a>, failure::Error> {
    trace!("gen_pool_context");

    let mut success_message = None;
    let mut error_message = None;

    if let Some(ref status_message) = *status_message {
        if status_message.kind() == "success" {
            success_message = Some(status_message.message());
        } else {
            error_message = Some(status_message.message());
        }
    }

    let unformatted_pools = database::get_pools(config, database)?;

    let pools = unformatted_pools.into_iter()
        .map(format_pool)
        .collect();

    Ok(PoolContext {
        pools,
        error_message,
        success_message,
    })
}

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
#[get("/editPools")]
pub fn get_edit_pools(
    config: &rocket::State<utils::types::Settings>,
    database: pool::DbConn,
    status_message: Option<rocket::request::FlashMessage<'_>>,
) -> Result<rocket_dyn_templates::Template, String> {
    trace!("get_edit_pools()");

    return match gen_pool_context(&*config, &*database, &status_message) {
        Ok(context) => {
            Ok(rocket_dyn_templates::Template::render(
                "edit_pools",
                &context,
            ))
        }
        Err(e) => {
            Err(e.to_string())
        }
    };
}

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
#[post("/addPools", data = "<pool_add>")]
pub fn post_add_pools(
    config: &rocket::State<utils::types::Settings>,
    database: pool::DbConn,
    pool_add: Form<models::PoolInsert>,
) -> rocket::response::Flash<rocket::response::Redirect> {
    trace!("post_add_pools()");

    let pool = pool_add.into_inner();
    debug!("pool: {:?}", pool);
    if let Err(errors) = pool.validate() {
        let errors = errors.field_errors();
        let msg = match find_first_validation_message(&errors) {
            Some(m) => m,
            None => "Failed to parse form data",
        };
        return rocket::response::Flash::error(
            rocket::response::Redirect::to("/editPools"),
            msg,
        );
    }
    let add_result = database::insert_pool(&*config, &*database, &pool);

    match add_result {
        Ok(0) | Err(_) => rocket::response::Flash::error(
            rocket::response::Redirect::to("/editPools"),
            "Failed to add pool",
        ),
        _ => rocket::response::Flash::success(
            rocket::response::Redirect::to("/editPools"),
            "Successfully added pool",
        ),
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
#[post("/deletePools", data = "<pool_edit>")]
pub fn post_delete_pools(
    config: &rocket::State<utils::types::Settings>,
    database: pool::DbConn,
    pool_edit: Form<models::PoolDelete>,
) -> rocket::response::Flash<rocket::response::Redirect> {
    trace!("post_delete_pools()");

    let pool = pool_edit.into_inner();
    debug!("pool: {:?}", pool);
    if let Err(errors) = pool.validate() {
        let errors = errors.field_errors();
        let msg = match find_first_validation_message(&errors) {
            Some(m) => m,
            None => "Failed to parse form data",
        };
        return rocket::response::Flash::error(
            rocket::response::Redirect::to("/editPools"),
            msg,
        );
    }
    let update_result = database::delete_pool(&*config, &*database, &pool);

    match update_result {
        Ok(0) | Err(_) => rocket::response::Flash::error(
            rocket::response::Redirect::to("/editPools"),
            "Failed to delete pool",
        ),
        _ => rocket::response::Flash::success(
            rocket::response::Redirect::to("/editPools"),
            "Successfully deleted pool",
        ),
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
#[post("/editPools", data = "<pool_edit>")]
pub fn post_edit_pools(
    config: &rocket::State<utils::types::Settings>,
    database: pool::DbConn,
    pool_edit: Form<models::PoolModify>,
) -> rocket::response::Flash<rocket::response::Redirect> {
    trace!("post_edit_pools()");

    let pool = pool_edit.into_inner();
    if let Err(errors) = pool.validate() {
        let errors = errors.field_errors();
        let msg = match find_first_validation_message(&errors) {
            Some(m) => m,
            None => "Failed to parse form data",
        };
        return rocket::response::Flash::error(
            rocket::response::Redirect::to("/editPools"),
            msg,
        );
    }
    let update_result = database::edit_pool(&*config, &*database, &pool);

    match update_result {
        Ok(0) | Err(_) => rocket::response::Flash::error(
            rocket::response::Redirect::to("/editPools"),
            "Failed to update pool",
        ),
        _ => rocket::response::Flash::success(
            rocket::response::Redirect::to("/editPools"),
            "Successfully updated pool",
        ),
    }
}

// customOwners (Exceptions)

#[derive(Serialize)]
struct PerCustomOwnerContext {
    custom_owner: models::CustomOwner,
    updated_at_local: String,
}

#[derive(Serialize, Default)]
struct CustomOwnerContext<'a> {
    custom_owners: Vec<PerCustomOwnerContext>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_message: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    success_message: Option<&'a str>,
}

fn format_custom_owner(custom_owner: models::CustomOwner) -> PerCustomOwnerContext {
    let updated_at_local = chrono::DateTime::<chrono::Local>::from_utc(
        custom_owner.updated_at,
        chrono::Local::now().offset().fix(),
    );
    trace!("format_custom_owner");

    let updated_at_local = format!("{}", updated_at_local.format("%F %r"));
    PerCustomOwnerContext {
        custom_owner,
        updated_at_local,
    }
}

fn gen_custom_owner_context<'a>(
    config: &utils::types::Settings,
    database: &database::DbConn,
    status_message: &'a Option<rocket::request::FlashMessage<'_>>,
) -> Result<CustomOwnerContext<'a>, failure::Error> {
    trace!("gen_custom_owner_context");

    let mut success_message = None;
    let mut error_message = None;

    if let Some(ref status_message) = *status_message {
        if status_message.kind() == "success" {
            success_message = Some(status_message.message());
        } else {
            error_message = Some(status_message.message());
        }
    }

    let unformatted_custom_owners = database::get_custom_owners(config, database)?;

    let custom_owners = unformatted_custom_owners.into_iter()
        .map(format_custom_owner)
        .collect();

    Ok(CustomOwnerContext {
        custom_owners,
        error_message,
        success_message,
    })
}


#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
#[get("/custom_owners/<name>")]
pub fn api_get_custom_owner(
    config: &rocket::State<utils::types::Settings>,
    database: pool::DbConn,
    name: String,
) -> Result<json::Json<models::CustomOwner>, rocket::response::status::Custom<String>> {
    trace!("api_get_custom_owner()");
    database::get_custom_owner(&*config, &*database, &name)
        .map_err(|_| {
            rocket::response::status::Custom(
                rocket::http::Status::InternalServerError,
                "500 Internal Server Error".to_string(),
            )
        })
        .and_then(|custom_owners| {
            custom_owners.ok_or_else(|| {
                rocket::response::status::Custom(
                    rocket::http::Status::NotFound,
                    "404 Not Found".to_string(),
                )
            })
        })
        .map(json::Json)
}

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
#[get("/custom_owners")]
pub fn api_get_custom_owners(
    config: &rocket::State<utils::types::Settings>,
    database: pool::DbConn,
) -> Result<json::Json<Vec<models::CustomOwner>>, String> {
    trace!("api_get_custom_owners()");
    return match database::get_custom_owners(&*config, &*database) {
        Ok(custom_owners) => {
            Ok(json::Json(custom_owners))
        }
        Err(e) => {
            Err(e.to_string())
        }
    };
}

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
#[get("/editCustomOwners")]
pub fn get_edit_custom_owners(
    config: &rocket::State<utils::types::Settings>,
    database: pool::DbConn,
    status_message: Option<rocket::request::FlashMessage<'_>>,
) -> Result<rocket_dyn_templates::Template, String> {
    trace!("get_edit_custom_owners()");

    match gen_custom_owner_context(&*config, &*database, &status_message) {
        Ok(context) => {
            Ok(rocket_dyn_templates::Template::render(
                "edit_custom_owners",
                &context,
            ))
        }
        Err(e) => {
            Err(e.to_string())
        }
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
#[post("/addCustomOwners", data = "<custom_owner_add>")]
pub fn post_add_custom_owners(
    config: &rocket::State<utils::types::Settings>,
    database: pool::DbConn,
    custom_owner_add: Form<models::CustomOwnerInsert>,
) -> rocket::response::Flash<rocket::response::Redirect> {
    trace!("post_add_custom_owners()");

    let mut custom_owner = custom_owner_add.into_inner();
    custom_owner.custom_owner_name = custom_owner.custom_owner_name.to_lowercase();
    custom_owner.recipient = custom_owner.recipient.to_lowercase();
    debug!("custom_owner: {:?}", custom_owner);
    if let Err(errors) = custom_owner.validate() {
        let errors = errors.field_errors();
        let msg = match find_first_validation_message(&errors) {
            Some(m) => m,
            None => "Failed to parse form data",
        };
        return rocket::response::Flash::error(
            rocket::response::Redirect::to("/editCustomOwners"),
            msg,
        );
    }

    let add_result = database::insert_custom_owner(&*config, &*database, &custom_owner);

    match add_result {
        Ok(0) | Err(_) => rocket::response::Flash::error(
            rocket::response::Redirect::to("/editCustomOwners"),
            "Failed to add custom_owner",
        ),
        _ => rocket::response::Flash::success(
            rocket::response::Redirect::to("/editCustomOwners"),
            "Successfully added custom_owner",
        ),
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
#[post("/deleteCustomOwners", data = "<custom_owner_edit>")]
pub fn post_delete_custom_owners(
    config: &rocket::State<utils::types::Settings>,
    database: pool::DbConn,
    custom_owner_edit: Form<models::CustomOwnerDelete>,
) -> rocket::response::Flash<rocket::response::Redirect> {
    trace!("post_delete_custom_owners()");

    let custom_owner = custom_owner_edit.into_inner();
    if let Err(errors) = custom_owner.validate() {
        let errors = errors.field_errors();
        let msg = match find_first_validation_message(&errors) {
            Some(m) => m,
            None => "Failed to parse form data",
        };
        return rocket::response::Flash::error(
            rocket::response::Redirect::to("/editCustomOwners"),
            msg,
        );
    }
    let update_result = database::delete_custom_owner(&*config, &*database, &custom_owner);

    match update_result {
        Ok(0) | Err(_) => rocket::response::Flash::error(
            rocket::response::Redirect::to("/editCustomOwners"),
            "Failed to delete custom_owner",
        ),
        _ => rocket::response::Flash::success(
            rocket::response::Redirect::to("/editCustomOwners"),
            "Successfully deleted custom_owner",
        ),
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
#[post("/editCustomOwners", data = "<custom_owner_edit>")]
pub fn post_edit_custom_owners(
    config: &rocket::State<utils::types::Settings>,
    database: pool::DbConn,
    custom_owner_edit: Form<models::CustomOwnerModify>,
) -> rocket::response::Flash<rocket::response::Redirect> {
    trace!("post_edit_custom_owners()");

    let mut custom_owner = custom_owner_edit.into_inner();
    custom_owner.custom_owner_name = custom_owner.custom_owner_name.to_lowercase();
    custom_owner.recipient = custom_owner.recipient.to_lowercase();
    if let Err(errors) = custom_owner.validate() {
        let errors = errors.field_errors();
        let msg = match find_first_validation_message(&errors) {
            Some(m) => m,
            None => "Failed to parse form data",
        };
        return rocket::response::Flash::error(
            rocket::response::Redirect::to("/editCustomOwners"),
            msg,
        );
    }
    let update_result = database::edit_custom_owner(&*config, &*database, &custom_owner);

    match update_result {
        Ok(0) | Err(_) => rocket::response::Flash::error(
            rocket::response::Redirect::to("/editCustomOwners"),
            "Failed to update custom_owner",
        ),
        _ => rocket::response::Flash::success(
            rocket::response::Redirect::to("/editCustomOwners"),
            "Successfully updated custom_owner",
        ),
    }
}

type ValidationErrorsInner =
    std::collections::HashMap<&'static str, Vec<validator::ValidationError>>;

fn find_first_validation_message<'a>(
    errors: &'a ValidationErrorsInner,
) -> Option<&'a std::borrow::Cow<'static, str>> {
    for es in errors {
        for e in es.1 {
            if let Some(ref e) = e.message {
                return Some(e);
            }
        }
    }
    None
}
