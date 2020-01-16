use crate::articles::article::Article;
use crate::authentication::token_handler::TokenHandler;
use crate::configuration::Configuration;
use crate::logger;
use serde_json::json;

static STATE_UNREAD: &str = "unread";
static TAG_UNTAG: &str = "_untagged_";

pub enum Action {
    Delete,
    Archive,
}

impl Action {
    fn value(&self) -> &str {
        match *self {
            Action::Delete => "delete",
            Action::Archive => "archive",
        }
    }
}

pub struct API {
    configuration: Configuration,
}

impl API {
    pub fn new() -> Self {
        Self {
            configuration: Default::default(),
        }
    }

    pub fn retrieve(&self) -> serde_json::Value {
        let token_handler = TokenHandler::new();
        let (consumer_key, pocket_retrieve_url, access_token) = (
            &self.configuration.consumer_key,
            &self.configuration.pocket_retrieve_url,
            &token_handler.read_auth(),
        );

        let params = [
            ("consumer_key", consumer_key),
            ("access_token", access_token),
            ("state", &STATE_UNREAD.to_owned()),
            ("tag", &TAG_UNTAG.to_owned()),
        ];
        let response = reqwest::Client::new()
            .post(pocket_retrieve_url)
            .form(&params)
            .send();

        match response {
            Ok(mut response) => {
                let response_text = response.text().unwrap();
                let json: serde_json::Value = serde_json::from_str(&response_text).unwrap();

                json.to_owned()
            }
            Err(_) => {
                logger::log("Could not retrieve Pocket's data");

                serde_json::Value::Null
            }
        }
    }

    pub fn modify(&self, articles: Vec<&Article>, action: Action) {
        let token_handler = TokenHandler::new();
        let (consumer_key, pocket_send_url, access_token) = (
            &self.configuration.consumer_key,
            &self.configuration.pocket_send_url,
            &token_handler.read_auth(),
        );

        let actions: serde_json::Value = articles
            .into_iter()
            .map(|article| {
                json!({
                    "action": action.value(),
                    "item_id": article.id,
                })
            })
            .collect();

        let params = [
            ("consumer_key", consumer_key),
            ("access_token", access_token),
            ("actions", &actions.to_string()),
        ];
        let response = reqwest::Client::new()
            .post(pocket_send_url)
            .form(&params)
            .send();

        match response {
            Ok(_) => {}
            Err(error) => {
                logger::log(&error.to_string());
            }
        }
    }

    pub fn delete(&self, articles: Vec<&Article>) {
        self.modify(articles, Action::Delete)
    }

    pub fn archive(&self, articles: Vec<&Article>) {
        self.modify(articles, Action::Archive)
    }
}
