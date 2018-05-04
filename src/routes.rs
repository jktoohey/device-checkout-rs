use database;
use failure;
use failure::ResultExt;
use models;
use rocket;
use rocket_contrib;
use std;
use utils;

#[get("/")]
pub fn index() -> rocket::response::Redirect {
    trace!("index()");
    rocket::response::Redirect::to("/devices")
}

#[get("/devices/<name>")]
pub fn api_get_device(
    config: rocket::State<utils::types::Settings>,
    name: String,
) -> Result<rocket_contrib::Json<models::Device>, rocket::response::status::Custom<String>> {
    trace!("api_get_device()");
    database::get_device(&*config, &name)
        .map_err(|_| {
            rocket::response::status::Custom(
                rocket::http::Status::InternalServerError,
                "500 Internal Server Error".to_string(),
            )
        })
        .and_then(|devices| {
            devices.ok_or(rocket::response::status::Custom(
                rocket::http::Status::NotFound,
                "404 Not Found".to_string(),
            ))
        })
        .map(rocket_contrib::Json)
}

#[get("/devices")]
pub fn api_get_devices(
    config: rocket::State<utils::types::Settings>,
) -> Result<rocket_contrib::Json<Vec<models::Device>>, failure::Error> {
    trace!("api_get_devices()");
    let devices = database::get_devices(&*config)?;
    Ok(rocket_contrib::Json(devices))
}

#[derive(Serialize)]
struct PerDeviceContext<'a> {
    device: models::Device,
    button_string: &'a str,
    button_class: &'a str,
}

#[derive(Serialize, Default)]
struct DevicesContext<'a> {
    devices: Vec<PerDeviceContext<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_message: Option<std::borrow::Cow<'a, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    success_message: Option<std::borrow::Cow<'a, str>>,
}

fn format_device<'a>(device: models::Device) -> PerDeviceContext<'a> {
    let button_string = match device.reservation_status {
        models::ReservationStatus::Reserved => "RETURN",
        _ => "CLAIM",
    };
    let button_class = match device.reservation_status {
        models::ReservationStatus::Reserved => "btn-danger",
        _ => "btn-primary",
    };
    PerDeviceContext {
        device: device,
        button_string,
        button_class,
    }
}

fn gen_device_context<'a, T>(
    config: &utils::types::Settings,
    db_result: &Option<Result<T, failure::Error>>,
) -> Result<DevicesContext<'a>, failure::Error> {
    trace!("gen_device_context");

    let mut success_message = None;
    let mut error_message = None;

    if let Some(db_result) = db_result {
        if let Ok(_) = db_result {
            success_message = Some("Device updated successufully".into());
        } else if let Err(e) = db_result {
            error_message = Some(format!("{}", e).into());
        }
    }

    let devices: Vec<_> = database::get_devices(config)?
        .into_iter()
        .map(format_device)
        .collect();

    Ok(DevicesContext {
        devices,
        error_message,
        success_message,
    })
}

#[get("/devices")]
pub fn get_devices(
    config: rocket::State<utils::types::Settings>,
) -> Result<rocket_contrib::Template, failure::Error> {
    trace!("get_devices()");

    let context = gen_device_context::<usize>(&*config, &None)?;
    Ok(rocket_contrib::Template::render("devices", &context))
}

#[get("/editDevices")]
pub fn get_edit_devices(
    config: rocket::State<utils::types::Settings>,
) -> Result<rocket_contrib::Template, failure::Error> {
    trace!("get_edit_devices()");

    let context = gen_device_context::<usize>(&*config, &None)?;
    Ok(rocket_contrib::Template::render("edit_devices", &context))
}

#[post("/editDevices", data = "<device_edit>")]
pub fn post_edit_devices(
    config: rocket::State<utils::types::Settings>,
    device_edit: rocket::request::Form<models::DeviceEdit>,
) -> Result<rocket_contrib::Template, failure::Error> {
    trace!("post_edit_devices()");

    let device = device_edit.get();
    let update_result = if device.save.is_some() {
        trace!("saving");
        database::edit_device(&*config, device)
            .context("Failed to save device")
            .map_err(|e| e.into())
    } else if device.delete.is_some() {
        trace!("deleting");
        database::delete_device(&*config, device)
            .context("Failed to delete device")
            .map_err(|e| e.into())
    } else if device.add.is_some() {
        trace!("adding");
        database::insert_device(
            &*config,
            &models::DeviceInsert {
                device_name: &device.device_name,
                device_url: &device.device_url,
            },
        ).context("Failed to add device")
            .map_err(|e| e.into())
    } else {
        Err(failure::err_msg("Unknown form action"))
    };

    let context = gen_device_context(&*config, &Some(update_result))?;
    Ok(rocket_contrib::Template::render("edit_devices", &context))
}

#[post("/devices", data = "<device_update>")]
pub fn post_devices(
    config: rocket::State<utils::types::Settings>,
    device_update: rocket::request::Form<models::DeviceUpdate>,
) -> Result<rocket_contrib::Template, failure::Error> {
    trace!("post_devices()");

    let mut device = device_update.into_inner();
    //toggle the reservation status
    if device.reservation_status == models::ReservationStatus::Available {
        device.reservation_status = models::ReservationStatus::Reserved;
    } else {
        device.reservation_status = models::ReservationStatus::Available;
    }

    //blank out the owner if we're returning it
    if device.reservation_status == models::ReservationStatus::Available {
        device.device_owner = None;
    }

    let update_result = database::update_device(&*config, &device)
        .context("Failed to save device")
        .map_err(|e| e.into());

    let context = gen_device_context(&*config, &Some(update_result))?;
    Ok(rocket_contrib::Template::render("devices", &context))
}
