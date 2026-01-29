use openapi::apis::configuration::{ApiKey, Configuration as OpenApiConfiguration};
use serde::{Deserialize, Serialize};

use crate::user::defaults::DEFAULT_API_ENDPOINT;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Configuration {
    pub post_auth_key: Option<String>,
    pub endpoint: Option<String>,
}

impl Configuration {
    pub fn to_openapi(self) -> OpenApiConfiguration {
        OpenApiConfiguration {
            base_path: self.endpoint.unwrap_or(DEFAULT_API_ENDPOINT.to_string()),
            api_key: self.post_auth_key.map(|key| ApiKey { prefix: None, key }),
            ..Default::default()
        }
    }
}
