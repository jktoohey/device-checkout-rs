use slack::api as slack_api;
use std::env;

pub struct SlackAPIClient {
    token: String,
    client: reqwest::blocking::Client,
}

#[cfg(not(test))]
pub fn slack_client_init() -> SlackAPIClient {
    let slack_client = SlackAPIClient {
        token: match env::var("SLACK_API_TOKEN") {
            Ok(token) => token,
            Err(_) => {
                error!("Failed to retrieve env var SLACK_API_TOKEN");
                "".to_string()
            }
        },
        client: slack_api::default_client().unwrap(),
    };
    return slack_client
}

#[cfg(not(test))]
// The Slack API doesn't have a method to retrieve a single channel by name
pub fn slack_channel_exists(test_name: &str, slack_client: &SlackAPIClient) -> bool {
    // slack-rs api does not support the new Slack conversations api
    let response = slack_client.client.get("https://slack.com/api/conversations.list")
        .query(&[
            ("token", &slack_client.token),
            ("limit", &500.to_string()),
            ("exclude_archived", &true.to_string()),
            ("types", &"public_channel,private_channel".to_string())])
        .send();
    match response {
        Ok(response) => {
            let json = serde_json::from_str::<slack_api::channels::ListResponse>(&response.text().unwrap()).unwrap();
            if let Some(channels) = &json.channels {
                for c in channels {
                    let id = &c.id.as_ref().unwrap();
                    let name = &c.name.as_ref().unwrap();
                    let is_channel = &c.is_channel.unwrap();
                    let is_archived = &c.is_archived.unwrap();
                    let is_private = &c.is_private.unwrap();
                    debug!("slack channel - id: {}, name: {}, is_channel: {}, is_archived: {}, is_private: {}", &id, &name, &is_channel, &is_archived, &is_private);
                    if *is_archived {
                        continue
                    }
                    if name.eq_ignore_ascii_case(&test_name) {
                        debug!("Slack channel matched name: {}", &test_name);
                        return true;
                    }
                }
            }
        },
        Err(error) => {
            warn!("Error occured while retrieving channels list: {:?}", error);
            warn!("slack_channel_exists() returns true if cannot reach Slack API.");
            return true
        },
    }
    return false;
}

#[cfg(not(test))]
// The Slack API doesn't have a method to retrieve a single user by name
pub fn slack_user_exists(test_name: &str, slack_client: &SlackAPIClient) -> bool {
    let users = slack_api::users::list(
        &slack_client.client,
        &slack_client.token,
        &slack_api::users::ListRequest::default(),
    );
    match users {
        Ok(users) => {
            if let Some(members) = &users.members {
                for u in members {
                    let id = &u.id.as_ref().unwrap();
                    let name = &u.name.as_ref().unwrap();
                    let profile = &u.profile.as_ref().unwrap();
                    let display_name = &profile.display_name.as_ref().unwrap();
                    let is_bot = &u.is_bot.unwrap();
                    let is_app_user = &u.is_app_user.unwrap();
                    let deleted = &u.deleted.unwrap();
                    debug!("id: {}, name: {}, display_name: {:?}, is_bot: {}, is_app_user: {}, deleted: {}", &id, &name, &display_name, &is_bot, &is_app_user, &deleted);
                    if *is_bot || *deleted {
                        continue
                    }
                    if name.eq_ignore_ascii_case(&test_name) {
                        debug!("Owner matched name: {}", &test_name);
                        return true;
                    }
                    if display_name.eq_ignore_ascii_case(&test_name) {
                        debug!("Owner matched display_name: {}", &test_name);
                        return true;
                    }
                }
            };
        },
        Err(error) => {
            warn!("Error occured while retrieving users list: {:?}", error);
            warn!("slack_user_exists() returns true if cannot reach Slack API.");
            return true
        },
    }
    return false;
}

#[cfg(test)]
pub fn slack_client_init() -> SlackAPIClient {
    SlackAPIClient {
        token: "new token".into(),
        client: slack_api::default_client().unwrap(),
    }
}

#[cfg(test)]
pub fn slack_channel_exists(test_name: &str, _slack_client: &SlackAPIClient) -> bool {
    if test_name.eq("slack_channel") {
        return true
    }
    return false
}

#[cfg(test)]
pub fn slack_user_exists(test_name: &str, _slack_client: &SlackAPIClient) -> bool {
    if test_name.eq("slack_user") {
        return true
    }
    return false
}
