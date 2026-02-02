use openapi::apis::configuration::{ApiKey, Configuration as OpenApiConfiguration};
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};

use crate::user::defaults::DEFAULT_API_ENDPOINT;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Configuration {
    pub post_auth_key: Option<String>,
    pub endpoint: Option<String>,
    pub vision_model_name: Option<String>,
    pub vision_model_key: Option<String>,
}

impl Configuration {
    pub fn to_openapi(self) -> OpenApiConfiguration {
        let mut headers = HeaderMap::new();
        let origin_extract = Regex::new(r"(https?:\/\/[\w.]+(?::\d+)?)\/?").unwrap();
        if let Some(origin) = origin_extract.captures(
            &self
                .endpoint
                .clone()
                .unwrap_or(DEFAULT_API_ENDPOINT.to_string()),
        ) {
            headers.insert(
                "Origin",
                HeaderValue::from_str(origin.get(1).unwrap().as_str()).unwrap(),
            );
        }
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();
        OpenApiConfiguration {
            base_path: self.endpoint.unwrap_or(DEFAULT_API_ENDPOINT.to_string()),
            api_key: self.post_auth_key.map(|key| ApiKey { prefix: None, key }),
            client,
            ..Default::default()
        }
    }
}
