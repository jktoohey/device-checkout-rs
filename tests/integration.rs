use rocket::local::blocking::{Client, LocalResponse};
use tempfile;
use victoria_dom;
use device_checkout::*;

#[test]
fn test_api_get_device() {
    let file = tempfile::NamedTempFile::new().expect("creating tempfile");
    let mut config = utils::types::Settings::new();
    config.database_url = file.path().to_string_lossy().to_owned().to_string();

    database::run_migrations(&config).expect("running migrations");

    let rocket = create_new_rocket(config);
    let client = Client::tracked(rocket).expect("valid rocket instance");
    let mut response = client.get("/api/devices/unit1").dispatch();
    assert_eq!(response.status(), rocket::http::Status::Ok);
    let body = response.into_string().unwrap();
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(v["device_url"], "http://unit1");

    let response = client.get("/api/devices/some_unknown_device").dispatch();
    assert_eq!(response.status(), rocket::http::Status::NotFound);
}

#[test]
fn test_api_get_devices() {
    let file = tempfile::NamedTempFile::new().expect("creating tempfile");
    let mut config = utils::types::Settings::new();
    config.database_url = file.path().to_string_lossy().to_owned().to_string();

    database::run_migrations(&config).expect("running migrations");

    let rocket = create_new_rocket(config);
    let client = Client::tracked(rocket).expect("valid rocket instance");
    let mut response = client.get("/api/devices").dispatch();
    assert_eq!(response.status(), rocket::http::Status::Ok);
    let body = response.into_string().unwrap();
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(v[0]["device_url"], "http://unit1");
    assert_eq!(v[1]["device_url"], "http://unit2");
}

#[test]
fn test_api_get_pools() {
    let file = tempfile::NamedTempFile::new().expect("creating tempfile");
    let mut config = utils::types::Settings::new();
    config.database_url = file.path().to_string_lossy().to_owned().to_string();

    database::run_migrations(&config).expect("running migrations");

    let rocket = create_new_rocket(config);
    let client = Client::tracked(rocket).expect("valid rocket instance");

    client.post("/addPools")
    .header(rocket::http::ContentType(rocket::http::MediaType::Form))
    .body(r#"pool_name=Custom1&description=test+description"#)
    .dispatch();

    client.post("/addPools")
    .header(rocket::http::ContentType(rocket::http::MediaType::Form))
    .body(r#"pool_name=Custom2&description=test+description+2"#)
    .dispatch();

    let mut response = client.get("/api/pools").dispatch();
    assert_eq!(response.status(), rocket::http::Status::Ok);
    let body = response.into_string().unwrap();
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(v[0]["pool_name"], "Default Pool");
    assert_eq!(v[0]["description"], "");
    assert_eq!(v[1]["pool_name"], "Custom1");
    assert_eq!(v[1]["description"], "test description");
    assert_eq!(v[2]["pool_name"], "Custom2");
    assert_eq!(v[2]["description"], "test description 2");
}

#[test]
fn test_api_get_custom_owner() {
    let file = tempfile::NamedTempFile::new().expect("creating tempfile");
    let mut config = utils::types::Settings::new();
    config.database_url = file.path().to_string_lossy().to_owned().to_string();

    database::run_migrations(&config).expect("running migrations");

    // Add custom owner record
    let rocket = create_new_rocket(config);
    let client = Client::tracked(rocket).expect("valid rocket instance");
    client.post("/addCustomOwners")
    .header(rocket::http::ContentType(rocket::http::MediaType::Form))
    .body(r#"custom_owner_name=Custom1&recipient=SlackUser&description=custom%20owner%20mapping%201"#)
    .dispatch();

    let mut response = client.get("/api/custom_owners/custom1").dispatch();
    assert_eq!(response.status(), rocket::http::Status::Ok);
    let body = response.into_string().unwrap();
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(v["id"], 1);
    assert_eq!(v["custom_owner_name"], "custom1");
    assert_eq!(v["recipient"], "slackuser");
    assert_eq!(v["description"], "custom owner mapping 1");

    // assert request is not case-sensitive
    let mut response = client.get("/api/custom_owners/CUSTOM1").dispatch();
    assert_eq!(response.status(), rocket::http::Status::Ok);
    let body = response.into_string().unwrap();
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(v["id"], 1);
    assert_eq!(v["custom_owner_name"], "custom1");
    assert_eq!(v["recipient"], "slackuser");
    assert_eq!(v["description"], "custom owner mapping 1");

    let response = client.get("/api/custom_owners/invalid").dispatch();
    assert_eq!(response.status(), rocket::http::Status::NotFound);
}

#[test]
fn test_api_get_custom_owners() {
    let file = tempfile::NamedTempFile::new().expect("creating tempfile");
    let mut config = utils::types::Settings::new();
    config.database_url = file.path().to_string_lossy().to_owned().to_string();

    database::run_migrations(&config).expect("running migrations");

    // Add custom owner records
    let rocket = create_new_rocket(config);
    let client = Client::tracked(rocket).expect("valid rocket instance");
    client.post("/addCustomOwners")
    .header(rocket::http::ContentType(rocket::http::MediaType::Form))
    .body(r#"custom_owner_name=Custom1&recipient=SlackUser&description=custom%20owner%20mapping%201"#)
    .dispatch();

    client.post("/addCustomOwners")
    .header(rocket::http::ContentType(rocket::http::MediaType::Form))
    .body(r#"custom_owner_name=Custom2&recipient=SlackChannel&description=custom%20owner%20mapping%202"#)
    .dispatch();

    client.post("/addCustomOwners")
    .header(rocket::http::ContentType(rocket::http::MediaType::Form))
    .body(r#"custom_owner_name=Custom3&recipient=None&description="#)
    .dispatch();

    let mut response = client.get("/api/custom_owners").dispatch();
    assert_eq!(response.status(), rocket::http::Status::Ok);
    let body = response.into_string().unwrap();
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(v[0]["id"], 1);
    assert_eq!(v[0]["custom_owner_name"], "custom1");
    assert_eq!(v[0]["recipient"], "slackuser");
    assert_eq!(v[0]["description"], "custom owner mapping 1");
    assert_eq!(v[1]["id"], 2);
    assert_eq!(v[1]["custom_owner_name"], "custom2");
    assert_eq!(v[1]["recipient"], "slackchannel");
    assert_eq!(v[1]["description"], "custom owner mapping 2");
    assert_eq!(v[2]["id"], 3);
    assert_eq!(v[2]["custom_owner_name"], "custom3");
    assert_eq!(v[2]["recipient"], "none");
    assert_eq!(v[2]["description"], "");
}

#[test]
fn test_api_delete_reservation() {
    let file = tempfile::NamedTempFile::new().expect("creating tempfile");
    let mut config = utils::types::Settings::new();
    config.database_url = file.path().to_string_lossy().to_owned().to_string();

    database::run_migrations(&config).expect("running migrations");
    let rocket = create_new_rocket(config);
    let client = Client::tracked(rocket).expect("valid rocket instance");

    /* TODO: Change this when the API for making reservations is ready. */
    let response = client
        .post("/devices")
        .header(rocket::http::ContentType(rocket::http::MediaType::Form))
        .body(r#"id=1&device_owner=Owner&comments=xyzzy&reservation_status=Available"#)
        .dispatch();
    let response = follow_redirect(&client, &response).unwrap();
    assert_eq!(response.status(), rocket::http::Status::Ok);

    let response = client.delete("/api/reservations/1").dispatch();
    assert_eq!(response.status(), rocket::http::Status::NoContent);

    /* Once a reservation has ended, you can't end it again. */
    let response = client.delete("/api/reservations/1").dispatch();
    assert_eq!(response.status(), rocket::http::Status::BadRequest);

    let response = client.delete("/api/reservations/9000").dispatch();
    assert_eq!(response.status(), rocket::http::Status::NotFound);
}

#[test]
fn test_api_post_reservations() {
    let file = tempfile::NamedTempFile::new().expect("creating tempfile");
    let mut config = utils::types::Settings::new();
    config.database_url = file.path().to_string_lossy().to_owned().to_string();

    database::run_migrations(&config).expect("running migrations");
    let rocket = create_new_rocket(config);
    let client = Client::tracked(rocket).expect("valid rocket instance");

    let mut response = client
        .post("/api/reservations")
        .header(rocket::http::ContentType::JSON)
        .body(r#"{"device_owner":"Barry","comments":"test reservation","device":{"pool_id":1}}"#)
        .dispatch();
    assert_eq!(response.status(), rocket::http::Status::Ok);
    let body = response.into_string().unwrap();
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(v["device_owner"], "Barry");
    assert_eq!(v["comments"], "test reservation");
    assert_eq!(v["device"]["reservation_status"], "Reserved");

    let response = client
        .post("/api/reservations")
        .header(rocket::http::ContentType::JSON)
        .body(r#"{"device_owner":"Barry","comments":"pool with no devices","device":{"pool_id":100}}"#)
        .dispatch();
    assert_eq!(response.status(), rocket::http::Status::NotFound);
}

#[test]
fn test_html_get_devices() {
    let file = tempfile::NamedTempFile::new().expect("creating tempfile");
    let mut config = utils::types::Settings::new();
    config.database_url = file.path().to_string_lossy().to_owned().to_string();

    database::run_migrations(&config).expect("running migrations");

    let rocket = create_new_rocket(config);
    let client = Client::tracked(rocket).expect("valid rocket instance");
    let mut response = client.get("/devices").dispatch();
    assert_eq!(response.status(), rocket::http::Status::Ok);
    let body = response.into_string().unwrap();
    let dom = victoria_dom::DOM::new(&body);
    let _ = dom
        .at(r#"a[href="http://unit1"]"#)
        .expect("failed to find unit1");
    let _ = dom
        .at(r#"a[href="http://unit2"]"#)
        .expect("failed to find unit2");
}

#[test]
fn test_html_get_edit_devices() {
    let file = tempfile::NamedTempFile::new().expect("creating tempfile");
    let mut config = utils::types::Settings::new();
    config.database_url = file.path().to_string_lossy().to_owned().to_string();

    database::run_migrations(&config).expect("running migrations");

    let rocket = create_new_rocket(config);
    let client = Client::tracked(rocket).expect("valid rocket instance");
    let mut response = client.get("/editDevices").dispatch();
    assert_eq!(response.status(), rocket::http::Status::Ok);
    let body = response.into_string().unwrap();
    let dom = victoria_dom::DOM::new(&body);
    let _ = dom
        .at(r#"input[name="device_url"][value="http://unit1"]"#)
        .expect("failed to find unit1");
    let _ = dom
        .at(r#"input[name="device_url"][value="http://unit2"]"#)
        .expect("failed to find unit2");
}

fn get_cookies(response: &LocalResponse<'_>) -> Vec<rocket::http::Cookie<'static>> {
    let mut cookies = Vec::new();
    for header in response.headers().get("Set-Cookie") {
        if let Ok(cookie) = rocket::http::Cookie::parse_encoded(header) {
            cookies.push(cookie.into_owned());
        }
    }
    cookies
}

fn get_redirect(response: &LocalResponse<'_>) -> Option<String> {
    if response.status() == rocket::http::Status::SeeOther {
        response
            .headers()
            .get("Location")
            .next()
            .map(|loc| loc.to_string())
    } else {
        None
    }
}

fn follow_redirect<'a>(
    client: &'a Client,
    response: &LocalResponse<'_>,
) -> Option<LocalResponse<'a>> {
    let cookies = get_cookies(&response);
    let location = match get_redirect(&response) {
        Some(l) => l,
        None => return None,
    };

    //manually follow the redirection with a new client
    let mut request = client.get(location);

    for cookie in cookies {
        request = request.cookie(cookie);
    }

    Some(request.dispatch())
}

#[test]
fn test_html_post_devices() {
    let file = tempfile::NamedTempFile::new().expect("creating tempfile");
    let mut config = utils::types::Settings::new();
    config.database_url = file.path().to_string_lossy().to_owned().to_string();

    database::run_migrations(&config).expect("running migrations");

    let rocket = create_new_rocket(config);
    let client = Client::tracked(rocket).expect("valid rocket instance");

    let response = client
        .post("/devices")
        .header(rocket::http::ContentType(rocket::http::MediaType::Form))
        .body(r#"id=1&device_owner=Owner&comments=xyzzy&reservation_status=Available"#)
        .dispatch();

    let mut response = follow_redirect(&client, &response).unwrap();
    assert_eq!(response.status(), rocket::http::Status::Ok);

    let body = response.into_string().unwrap();
    let dom = victoria_dom::DOM::new(&body);

    let _ = dom
        .at(r#"#success_message"#)
        .expect("failed to find success message");
    assert!(dom.at(r#"#error_message"#).is_none());

    let _ = dom
        .at(r#"input[name="device_owner"][value="Owner"]"#)
        .expect("failed to find owner");

    let _ = dom
        .at(r#"input[name="reservation_status"][value="Reserved"]"#)
        .expect("failed to find reservation status");

    let _ = dom
        .at(r#"input[name="comments"][value="xyzzy"]"#)
        .expect("failed to find comments");
}

#[test]
fn test_html_reserve_without_user() {
    let file = tempfile::NamedTempFile::new().expect("creating tempfile");
    let mut config = utils::types::Settings::new();
    config.database_url = file.path().to_string_lossy().to_owned().to_string();

    database::run_migrations(&config).expect("running migrations");

    let rocket = create_new_rocket(config);
    let client = Client::tracked(rocket).expect("valid rocket instance");

    let response = client
        .post("/devices")
        .header(rocket::http::ContentType(rocket::http::MediaType::Form))
        .body(r#"id=1&device_owner=&comments=xyzzy&reservation_status=Available"#)
        .dispatch();

    let mut response = follow_redirect(&client, &response).unwrap();
    assert_eq!(response.status(), rocket::http::Status::Ok);

    let body = response.into_string().unwrap();
    let dom = victoria_dom::DOM::new(&body);

    let _ = dom
        .at(r#"#error_message"#)
        .expect("failed to find error message");
    assert!(dom.at(r#"#success_message"#).is_none());
}

#[test]
fn test_html_edit_devices() {
    let file = tempfile::NamedTempFile::new().expect("creating tempfile");
    let mut config = utils::types::Settings::new();
    config.database_url = file.path().to_string_lossy().to_owned().to_string();

    database::run_migrations(&config).expect("running migrations");

    let rocket = create_new_rocket(config);
    let client = Client::tracked(rocket).expect("valid rocket instance");

    let response = client
        .post("/editDevices")
        .header(rocket::http::ContentType(rocket::http::MediaType::Form))
        .body(r#"id=1&device_name=testunit&device_url=http://testurl&pool_id=1&save=SAVE"#)
        .dispatch();

    let mut response = follow_redirect(&client, &response).unwrap();
    assert_eq!(response.status(), rocket::http::Status::Ok);
    let body = response.into_string().unwrap();

    let dom = victoria_dom::DOM::new(&body);
    let _ = dom
        .at(r#"#success_message"#)
        .expect("failed to find success message");
    assert!(dom.at(r#"#error_message"#).is_none());

    let _ = dom
        .at(r#"input[name="device_name"][value="testunit"]"#)
        .expect("failed to find edited device name");

    let _ = dom
        .at(r#"input[name="device_url"][value="http://testurl"]"#)
        .expect("failed to find edited device url");
}

#[test]
fn test_html_edit_devices_bad_url() {
    let file = tempfile::NamedTempFile::new().expect("creating tempfile");
    let mut config = utils::types::Settings::new();
    config.database_url = file.path().to_string_lossy().to_owned().to_string();

    database::run_migrations(&config).expect("running migrations");

    let rocket = create_new_rocket(config);
    let client = Client::tracked(rocket).expect("valid rocket instance");

    let response = client
        .post("/editDevices")
        .header(rocket::http::ContentType(rocket::http::MediaType::Form))
        .body(r#"id=1&device_name=testunit&device_url=notaurl&save=SAVE"#)
        .dispatch();

    assert_eq!(response.status(), rocket::http::Status::UnprocessableEntity);
}

#[test]
fn test_html_edit_devices_delete() {
    let file = tempfile::NamedTempFile::new().expect("creating tempfile");
    let mut config = utils::types::Settings::new();
    config.database_url = file.path().to_string_lossy().to_owned().to_string();

    database::run_migrations(&config).expect("running migrations");

    let rocket = create_new_rocket(config);
    let client = Client::tracked(rocket).expect("valid rocket instance");

    let response = client
        .post("/deleteDevices")
        .header(rocket::http::ContentType(rocket::http::MediaType::Form))
        .body(r#"id=1&device_name=testunit&device_url=testurl&delete=DELETE"#)
        .dispatch();

    let mut response = follow_redirect(&client, &response).unwrap();
    assert_eq!(response.status(), rocket::http::Status::Ok);
    let body = response.into_string().unwrap();

    let dom = victoria_dom::DOM::new(&body);

    let _ = dom
        .at(r#"#success_message"#)
        .expect("failed to find success message");
    assert!(dom.at(r#"#error_message"#).is_none());

    assert!(dom.at(r#"form[name="edit-1"]"#).is_none());
}

#[test]
fn test_html_add_devices() {
    let file = tempfile::NamedTempFile::new().expect("creating tempfile");
    let mut config = utils::types::Settings::new();
    config.database_url = file.path().to_string_lossy().to_owned().to_string();

    database::run_migrations(&config).expect("running migrations");

    let rocket = create_new_rocket(config);
    let client = Client::tracked(rocket).expect("valid rocket instance");

    let response = client
        .post("/addDevices")
        .header(rocket::http::ContentType(rocket::http::MediaType::Form))
        .body(r#"device_name=testunit&device_url=http://testurl&pool_id=1&add=ADD"#)
        .dispatch();

    let mut response = follow_redirect(&client, &response).unwrap();
    assert_eq!(response.status(), rocket::http::Status::Ok);
    let body = response.into_string().unwrap();

    let dom = victoria_dom::DOM::new(&body);

    let _ = dom
        .at(r#"#success_message"#)
        .expect("failed to find success message");
    assert!(dom.at(r#"#error_message"#).is_none());

    let _ = dom
        .at(r#"input[name="device_url"][value="http://testurl"]"#)
        .expect("failed to find added device");
}

#[test]
fn test_html_add_devices_bad_url() {
    let file = tempfile::NamedTempFile::new().expect("creating tempfile");
    let mut config = utils::types::Settings::new();
    config.database_url = file.path().to_string_lossy().to_owned().to_string();

    database::run_migrations(&config).expect("running migrations");

    let rocket = create_new_rocket(config);
    let client = Client::tracked(rocket).expect("valid rocket instance");

    let response = client
        .post("/addDevices")
        .header(rocket::http::ContentType(rocket::http::MediaType::Form))
        .body(r#"device_name=testunit&device_url=notaurl&add=ADD"#)
        .dispatch();

    assert_eq!(response.status(), rocket::http::Status::UnprocessableEntity);
}

#[test]
fn test_get_root() {
    let file = tempfile::NamedTempFile::new().expect("creating tempfile");
    let mut config = utils::types::Settings::new();
    config.database_url = file.path().to_string_lossy().to_owned().to_string();

    database::run_migrations(&config).expect("running migrations");

    let rocket = create_new_rocket(config);
    let client = Client::tracked(rocket).expect("valid rocket instance");

    let response = client.get("/").dispatch();

    assert_eq!(response.status(), rocket::http::Status::SeeOther);
}

#[test]
fn test_reserve_already_reserved() {
    let file = tempfile::NamedTempFile::new().expect("creating tempfile");
    let mut config = utils::types::Settings::new();
    config.database_url = file.path().to_string_lossy().to_owned().to_string();

    database::run_migrations(&config).expect("running migrations");

    let rocket = create_new_rocket(config);
    let client = Client::tracked(rocket).expect("valid rocket instance");

    //reserve unit1
    let response = client
        .post("/devices")
        .header(rocket::http::ContentType(rocket::http::MediaType::Form))
        .body(r#"id=1&device_owner=Owner&comments=xyzzy&reservation_status=Available"#)
        .dispatch();

    let response = follow_redirect(&client, &response).unwrap();
    assert_eq!(response.status(), rocket::http::Status::Ok);
    let body = response.into_string().unwrap();

    let dom = victoria_dom::DOM::new(&body);
    let _ = dom
        .at(r#"input[name="reservation_status"][value="Reserved"]"#)
        .expect("failed to find reservation status");

    //reserve unit2
    let response = client
        .post("/devices")
        .header(rocket::http::ContentType(rocket::http::MediaType::Form))
        .body(r#"id=1&device_owner=Owner2&comments=xyzzy&reservation_status=Available"#)
        .dispatch();

    let response = follow_redirect(&client, &response).unwrap();
    assert_eq!(&response.status(), &rocket::http::Status::Ok);
    let body = &response.into_string().unwrap();

    let dom = victoria_dom::DOM::new(&body);

    let _ = dom
        .at(r#"#error_message"#)
        .expect("failed to find error message");
    assert!(dom.at(r#"#success_message"#).is_none());

    let _ = dom
        .at(r#"input[name="device_owner"][value="Owner"]"#)
        .expect("failed to find owner");
    assert!(dom
        .at(r#"input[name="device_owner"][value="Owner2"]"#)
        .is_none());
}

#[test]
fn test_returning_clears_fields() {
    let file = tempfile::NamedTempFile::new().expect("creating tempfile");
    let mut config = utils::types::Settings::new();
    config.database_url = file.path().to_string_lossy().to_owned().to_string();

    database::run_migrations(&config).expect("running migrations");

    let rocket = create_new_rocket(config);
    let client = Client::tracked(rocket).expect("valid rocket instance");

    //reserve unit1
    let response = client
        .post("/devices")
        .header(rocket::http::ContentType(rocket::http::MediaType::Form))
        .body(r#"id=1&device_owner=Owner&comments=xyzzy&reservation_status=Available"#)
        .dispatch();

    let response = follow_redirect(&client, &response).unwrap();
    assert_eq!(response.status(), rocket::http::Status::Ok);

    //return unit1
    let response = client
        .post("/devices")
        .header(rocket::http::ContentType(rocket::http::MediaType::Form))
        .body(r#"id=1&reservation_status=Reserved"#)
        .dispatch();

    let mut response = follow_redirect(&client, &response).unwrap();
    assert_eq!(response.status(), rocket::http::Status::Ok);
    let body = response.into_string().unwrap();

    let dom = victoria_dom::DOM::new(&body);

    let _ = dom
        .at(r#"#success_message"#)
        .expect("failed to find success message");
    assert!(dom.at(r#"#error_message"#).is_none());

    //test that the old values for the reservation are gone
    assert!(dom
        .at(r#"input[name="device_owner"][value="Owner"]"#)
        .is_none());
    assert!(dom.at(r#"input[name="comments"][value="xyzzy"]"#).is_none());

    //but that they still exist
    let _ = dom
        .at(r#"input[name="device_owner"][form="reserve-1"][value]"#)
        .expect("failed to find empty owner");
    let _ = dom
        .at(r#"input[name="comments"][form="reserve-1"][value]"#)
        .expect("failed to find empty comments");
}
