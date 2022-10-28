use chrono;
use rocket;
use crate::database;
use crate::schema::*;
use crate::utils;
use crate::utils::slack;
use std;
use validator::{Validate, ValidationError};

#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Serialize, Deserialize, DbEnum, FromFormField
)]
pub enum ReservationStatus {
    Available,
    Reserved,
}

impl Default for ReservationStatus {
    fn default() -> Self {
        ReservationStatus::Available
    }
}

impl std::ops::Not for ReservationStatus {
    type Output = ReservationStatus;

    fn not(self) -> Self::Output {
        match self {
            ReservationStatus::Available => ReservationStatus::Reserved,
            ReservationStatus::Reserved => ReservationStatus::Available,
        }
    }
}

//deliberately not making this Copy
#[derive(
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Hash,
    Identifiable,
    Queryable,
    Associations,
    Serialize,
    Deserialize,
)]
#[belongs_to(Pool)]
pub struct Device {
    pub id: i32,
    pub device_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub device_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub device_owner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub comments: Option<String>,
    pub reservation_status: ReservationStatus,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub pool_id: i32,
}

#[cfg_attr(
    feature = "cargo-clippy",
    allow(print_literal, suspicious_else_formatting)
)]
#[derive(
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Default,
    Clone,
    Hash,
    Serialize,
    Deserialize,
    FromForm,
    Validate,
)]
#[validate(schema(function = "validate_device_checkout"))]
pub struct DeviceUpdate {
    pub id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub device_owner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub comments: Option<String>,
    pub reservation_status: ReservationStatus,
}

fn validate_device_checkout(device: &DeviceUpdate) -> Result<(), ValidationError> {
    if device.reservation_status == ReservationStatus::Reserved {
        debug!("Validate device (id: {}) reserved - owner is valid", &device.id);
        match device.device_owner {
            Some(ref owner) if !owner.trim().is_empty() => {
                let slack_client = slack::slack_client_init();
                let slack_user_exists = slack::slack_user_exists(&owner.trim(), &slack_client);

                let mut config = utils::cmdline::parse_cmdline();
                config.module_path = Some(module_path!().into());
                let database = database::establish_connection(&config).unwrap();
                let is_custom_owner = match database::get_custom_owner(&config, &database, &owner.trim()) {
                    Ok(Some(custom_owner)) => {
                        trace!("User in custom owners: {:?}", custom_owner);
                        debug!("Matched owner '{}' to custom owner '{}'", &owner.trim(), &custom_owner.custom_owner_name);
                        true
                    },
                    _ => {
                        debug!("Owner '{}' is not in custom owners", &owner.trim());
                        false
                    }
                };
                if slack_user_exists || is_custom_owner {
                    Ok(())
                } else {
                    let mut e = ValidationError::new("reservation");
                    e.message = Some("Please enter a valid slack username or custom owner when reserving a device.".into());
                    Err(e)
                }
            },
            _ => {
                let mut e = ValidationError::new("reservation");
                e.message = Some("Please supply a username when reserving a device".into());
                Err(e)
            }
        }
    } else {
        debug!("Validate device (id: {}) available", &device.id);
        Ok(())
    }
}

#[cfg_attr(
    feature = "cargo-clippy",
    allow(print_literal, suspicious_else_formatting)
)]
#[derive(
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Default,
    Clone,
    Hash,
    Serialize,
    Deserialize,
    FromForm,
    Validate,
)]
pub struct DeviceEdit {
    pub id: i32,
    #[validate(length(min = "1", message = "Device names cannot be empty"))]
    pub device_name: String,
    #[validate(url(message = "URL was invalid"))]
    pub device_url: String,
    pub pool_id: i32,
}

#[cfg_attr(
    feature = "cargo-clippy",
    allow(print_literal, suspicious_else_formatting)
)]
#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, Default, Clone, Hash, Serialize, Deserialize, FromForm,
)]
pub struct DeviceDelete {
    pub id: i32,
}

#[cfg_attr(
    feature = "cargo-clippy",
    allow(print_literal, suspicious_else_formatting)
)]
#[derive(
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Default,
    Clone,
    Hash,
    Serialize,
    Deserialize,
    FromForm,
    Insertable,
    Validate,
)]
#[table_name = "devices"]
pub struct DeviceInsert {
    #[validate(length(min = "1", message = "Device names cannot be empty"))]
    pub device_name: String,
    #[validate(url(message = "URL was invalid"))]
    pub device_url: String,
    pub pool_id: i32,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Queryable, Serialize, Deserialize)]
pub struct Reservation {
    pub id: i32,
    pub device_owner: String,
    pub comments: Option<String>,
    pub device: Device,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Serialize, Deserialize)]
pub struct ReservationRequest {
    pub device_owner: Option<String>,
    pub comments: Option<String>,
    pub device: ReservationRequestDevice,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Serialize, Deserialize)]
pub struct ReservationRequestDevice {
    pub id: Option<i32>,
    pub device_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub device_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub device_owner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub comments: Option<String>,
    pub pool_id: i32,
}

// pools
#[derive(
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Hash,
    Identifiable,
    Queryable,
    Associations,
    Serialize,
    Deserialize,
)]
pub struct Pool {
    pub id: i32,
    pub pool_name: String,
    pub description: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[cfg_attr(
    feature = "cargo-clippy",
    allow(print_literal, suspicious_else_formatting)
)]
#[derive(
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Default,
    Clone,
    Hash,
    Serialize,
    Deserialize,
    FromForm,
    Validate,
)]
pub struct PoolModify {
    pub id: i32,
    #[validate(length(min = "1", message = "pool_name cannot be empty"))]
    pub pool_name: String,
    pub description: Option<String>,
}

#[cfg_attr(
    feature = "cargo-clippy",
    allow(print_literal, suspicious_else_formatting)
)]
#[derive(
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Default,
    Clone,
    Hash,
    Serialize,
    Deserialize,
    Insertable,
    FromForm,
    Validate,
)]
#[table_name = "pools"]
// We have a separate struct for insert because rocket expects the form to match exactly
pub struct PoolInsert {
    #[validate(length(min = "1", message = "pool_name cannot be empty"))]
    pub pool_name: String,
    pub description: Option<String>,
}

#[cfg_attr(
    feature = "cargo-clippy",
    allow(print_literal, suspicious_else_formatting)
)]
#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, Default, Clone, Hash, Serialize, Deserialize, FromForm, Validate
)]
#[validate(schema(function = "validate_pool_delete"))]
pub struct PoolDelete {
    pub id: i32,
}

fn validate_pool_delete(pool: &PoolDelete) -> Result<(), ValidationError> {
    debug!("Validate pool (id: {}) delete - not default pool", &pool.id);
    let ref pool_id = pool.id;
    if *pool_id == 1 {
        let mut e = ValidationError::new("pool");
        e.message = Some("Default pool cannot be deleted".into());
        return Err(e);
    }

    // delete allowed only if pool is empty
    let mut config = utils::cmdline::parse_cmdline();
    config.module_path = Some(module_path!().into());
    let database = database::establish_connection(&config).unwrap();
    debug!("Validate pool (id: {}) delete - pool is empty", &pool.id);
    let pool_devices = database::get_devices_in_pool(&config, &database, *pool_id)
        .expect("Failed to get pool devices");
    if pool_devices.iter().count() > 0 {
        let mut e = ValidationError::new("pool");
        e.message = Some("Cannot delete non-empty pool".into());
        return Err(e);
    }
    Ok(())
}

// custom owners

#[derive(
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Hash,
    Identifiable,
    Queryable,
    Associations,
    Serialize,
    Deserialize,
)]
pub struct CustomOwner {
    pub id: i32,
    pub custom_owner_name: String,
    pub recipient: String,
    pub description: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

#[cfg_attr(
    feature = "cargo-clippy",
    allow(print_literal, suspicious_else_formatting)
)]
#[derive(
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Default,
    Clone,
    Hash,
    Serialize,
    Deserialize,
    FromForm,
    Validate,
)]
#[validate(schema(function = "validate_custom_owner_modify"))]
pub struct CustomOwnerModify {
    pub id: i32,
    #[validate(length(min = "1", message = "custom_owner_name cannot be empty"))]
    pub custom_owner_name: String,
    #[validate(length(min = "1", message = "recipient cannot be empty"))]
    pub recipient: String,
    pub description: Option<String>,
}

fn validate_custom_owner_modify(custom_owner: &CustomOwnerModify) -> Result<(), ValidationError> {
    let mut config = utils::cmdline::parse_cmdline();
    config.module_path = Some(module_path!().into());
    let database = database::establish_connection(&config).unwrap();
    // modify name allowed only if no devices reserved
    debug!("Validate custom_owner (id: {}) modify name - not in reserved devices", &custom_owner.id);
    let custom_owner_rec = database::get_custom_owner_by_id(&config, &database, custom_owner.id);
    match custom_owner_rec {
        Ok(Some(custom_owner_rec)) => {
            if custom_owner_rec.custom_owner_name.ne(&custom_owner.custom_owner_name) {
                let custom_owner_devices = database::get_devices_by_owner(&config, &database, &custom_owner_rec.custom_owner_name)
                    .expect("Failed to get custom_owner devices");
                if custom_owner_devices.iter().count() > 0 {
                    let mut e = ValidationError::new("custom_owner");
                    e.message = Some("Cannot modify name of custom_owner with devices reserved".into());
                    return Err(e);
                }
            }
        },
        Ok(None) => {
            #[cfg(test)]
            warn!("custom_owner_rec not found. This is acceptable in unit tests.");
        },
        Err(error) => error!("Error occured while retrieving custom_owner_rec: {:?}", error),
    }
    // validate recipient
    debug!("Validate custom_owner (id: {}) modify - recipient: {}", &custom_owner.id, &custom_owner.recipient);
    let ref recipient = custom_owner.recipient;
    if recipient.eq_ignore_ascii_case("none".into()) {
        debug!("Recipient '{}' matches \"none\"", &custom_owner.recipient);
        Ok(())
    } else {
        let slack_client = slack::slack_client_init();
        let slack_user_exists = slack::slack_user_exists(&recipient.trim(), &slack_client);
        let slack_channel_exists = slack::slack_channel_exists(&recipient.trim(), &slack_client);
        if slack_user_exists || slack_channel_exists {
            Ok(())
        } else {
            let mut e = ValidationError::new("custom_owner");
            e.message = Some("Recipient must be valid Slack user, Slack channel or \"None\".".into());
            Err(e)
        }
    }
}

#[cfg_attr(
    feature = "cargo-clippy",
    allow(print_literal, suspicious_else_formatting)
)]
#[derive(
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Default,
    Clone,
    Hash,
    Serialize,
    Deserialize,
    Insertable,
    FromForm,
    Validate,
)]
#[validate(schema(function = "validate_custom_owner_insert"))]
#[table_name = "custom_owners"]
// We have a separate struct for insert because rocket expects the form to match exactly
pub struct CustomOwnerInsert {
    #[validate(length(min = "1", message = "custom_owner_name cannot be empty"))]
    pub custom_owner_name: String,
    #[validate(length(min = "1", message = "recipient cannot be empty"))]
    pub recipient: String,
    pub description: Option<String>,
}

fn validate_custom_owner_insert(custom_owner: &CustomOwnerInsert) -> Result<(), ValidationError> {
    debug!("Validate custom_owner insert - recipient: {}", &custom_owner.recipient);
    let ref recipient = custom_owner.recipient;
    if recipient.eq_ignore_ascii_case("none".into()) {
        debug!("Recipient '{}' matches \"none\"", &custom_owner.recipient);
        Ok(())
    } else {
        let slack_client = slack::slack_client_init();
        let slack_user_exists = slack::slack_user_exists(&recipient.trim(), &slack_client);
        let slack_channel_exists = slack::slack_channel_exists(&recipient.trim(), &slack_client);
        if slack_user_exists || slack_channel_exists {
            Ok(())
        } else {
            let mut e = ValidationError::new("custom_owner");
            e.message = Some("Recipient must be valid Slack user, Slack channel or \"none\".".into());
            Err(e)
        }
    }
}

#[cfg_attr(
    feature = "cargo-clippy",
    allow(print_literal, suspicious_else_formatting)
)]
#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, Default, Clone, Hash, Serialize, Deserialize, FromForm, Validate
)]
#[validate(schema(function = "validate_custom_owner_delete"))]
pub struct CustomOwnerDelete {
    pub id: i32,
}

fn validate_custom_owner_delete(custom_owner: &CustomOwnerDelete) -> Result<(), ValidationError> {
    let mut config = utils::cmdline::parse_cmdline();
    config.module_path = Some(module_path!().into());
    let database = database::establish_connection(&config).unwrap();
    // delete allowed only if no devices reserved
    debug!("Validate custom_owner (id: {}) delete - not in reserved devices", &custom_owner.id);
    let ref custom_owner_id = custom_owner.id;
    let custom_owner_rec = database::get_custom_owner_by_id(&config, &database, *custom_owner_id)
        .expect("Failed to get custom owner by id").unwrap();
    let custom_owner_devices = database::get_devices_by_owner(&config, &database, &custom_owner_rec.custom_owner_name)
        .expect("Failed to get custom_owner devices");
    if custom_owner_devices.iter().count() > 0 {
        let mut e = ValidationError::new("custom_owner");
        e.message = Some("Cannot delete custom_owner with devices reserved".into());
        return Err(e);
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_device_update_validation() {
        let mut device = DeviceUpdate {
            id: 3,
            device_owner: None,
            comments: None,
            reservation_status: ReservationStatus::Available,
        };
        assert!(device.validate().is_ok()); // empty fields valid if device being returned
        device.reservation_status = ReservationStatus::Reserved;
        assert!(device.validate().is_err()); // empty device_owner not ok
        device.device_owner = Some("fake_user".into());
        assert!(device.validate().is_err()); // invalid slack user not ok
        device.device_owner = Some("slack_user".into());
        assert!(device.validate().is_ok()); // slack user valid
        // TODO custom_owner db entry required
        // device.device_owner = Some("custom1".into());
        // assert!(device.validate().is_ok()); // custom owner valid
    }

    #[test]
    fn test_device_insert_validation() {
        let mut device = DeviceInsert {
            device_name: "".into(),
            device_url: "".into(),
            pool_id: 0,
        };
        assert!(device.validate().is_err());
        device.device_name = "test".into();
        device.device_url = "http://test".into();
        assert!(device.validate().is_ok());
        device.device_name = "".into();
        assert!(device.validate().is_err());
    }

    #[test]
    fn test_device_edit_validation() {
        let mut device = DeviceEdit {
            id: 0,
            device_name: "".into(),
            device_url: "".into(),
            pool_id: 0,
        };
        assert!(device.validate().is_err());
        device.device_name = "test".into();
        device.device_url = "http://test".into();
        assert!(device.validate().is_ok());
        device.device_name = "".into();
        assert!(device.validate().is_err());
    }

    #[test]
    fn test_pool_insert_validation() {
        let mut pool = PoolInsert {
            pool_name: "custom1".into(),
            description: Some("test description".into()),
        };
        assert!(pool.validate().is_ok()); // normal case ok
        pool.description = Some("".into());
        assert!(pool.validate().is_ok()); // empty pool description is ok
        pool.pool_name = "".into();
        assert!(pool.validate().is_err()); // empty name not ok
    }

    #[test]
    fn test_pool_edit_validation() {
        let mut pool = PoolModify {
            id: 0,
            pool_name: "custom1".into(),
            description: Some("test description".into()),
        };
        assert!(pool.validate().is_ok()); // normal case is ok
        pool.description = Some("".into());
        assert!(pool.validate().is_ok()); // empty pool description is ok
        pool.pool_name = "".into();
        assert!(pool.validate().is_err()); // empty name not ok
    }

    #[test]
    fn test_custom_owner_insert_validation() {
        let mut custom_owner = CustomOwnerInsert {
            custom_owner_name: "custom1".into(),
            recipient: "slack_channel".into(),
            description: Some("description".into()),
        };
        assert!(custom_owner.validate().is_ok()); // slack channel recipient is ok
        custom_owner.recipient = "slack_user".into();
        assert!(custom_owner.validate().is_ok()); // slack user recipient is ok
        custom_owner.recipient = "none".into();
        assert!(custom_owner.validate().is_ok()); // "none" recipient is ok
        custom_owner.custom_owner_name = "".into();
        assert!(custom_owner.validate().is_err()); // empty name not ok
        custom_owner.custom_owner_name = "test".into();
        custom_owner.recipient = "".into();
        assert!(custom_owner.validate().is_err()); // empty recipient not ok
        custom_owner.recipient = "not_real".into();
        assert!(custom_owner.validate().is_err()); // invalid recipient not ok
        custom_owner.recipient = "slack_channel".into();
        custom_owner.description = Some("".into());
        assert!(custom_owner.validate().is_ok()); // empty description is ok
        custom_owner.description = None;
        assert!(custom_owner.validate().is_ok()); // None description is ok
    }

    #[test]
    fn test_custom_owner_edit_validation() {
        let mut custom_owner = CustomOwnerModify {
            id: 0,
            custom_owner_name: "custom1".into(),
            recipient: "slack_channel".into(),
            description: Some("description".into()),
        };
        assert!(custom_owner.validate().is_ok()); // slack channel recipient is ok
        custom_owner.recipient = "slack_user".into();
        assert!(custom_owner.validate().is_ok()); // slack user recipient is ok
        custom_owner.recipient = "none".into();
        assert!(custom_owner.validate().is_ok()); // "none" recipient is ok
        custom_owner.custom_owner_name = "".into();
        assert!(custom_owner.validate().is_err()); // empty name not ok
        custom_owner.custom_owner_name = "test".into();
        custom_owner.recipient = "".into();
        assert!(custom_owner.validate().is_err()); // empty recipient not ok
        custom_owner.recipient = "not_real".into();
        assert!(custom_owner.validate().is_err()); // invalid recipient not ok
        custom_owner.recipient = "slack_channel".into();
        custom_owner.description = Some("".into());
        assert!(custom_owner.validate().is_ok()); // empty description is ok
        custom_owner.description = None;
        assert!(custom_owner.validate().is_ok()); // None description is ok
    }
}
