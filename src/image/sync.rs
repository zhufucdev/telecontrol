use openapi::{
    apis::{
        configuration::Configuration,
        default_api::{self, ImagePostError, ImagePutError},
    },
    models::ImagePutRequest,
};
use strum::Display;
use tokio::fs;

use crate::image::{Image, ImageSource};

fn urlencode<T: AsRef<str>>(s: T) -> String {
    urlencoding::encode(s.as_ref()).to_string()
}

impl Image {
    pub async fn push(&self, configuration: &Configuration) -> Result<i32, Error> {
        Ok(match self.source.clone() {
            ImageSource::LocalFile(path_buf) => {
                let content_length = fs::metadata(&path_buf)
                    .await
                    .map(|metadata| metadata.len() as i32)?;
                let response = default_api::image_post(
                    configuration,
                    &urlencode(&self.alt_text),
                    &urlencode(self.source.filename().unwrap_or("unknown".to_string())),
                    content_length,
                    path_buf,
                )
                .await?;
                response.id
            }
            ImageSource::Url(url) => {
                let req = ImagePutRequest::new(url, self.alt_text.clone());
                let id = default_api::image_put(configuration, req).await?;
                id
            }
        })
    }
}

#[derive(Debug, Display)]
pub enum Error {
    #[strum(to_string = "upload: {0}")]
    Upload(openapi::apis::Error<ImagePostError>),
    #[strum(to_string = "create: {0}")]
    Create(openapi::apis::Error<ImagePutError>),
    #[strum(to_string = "handle files: {0}")]
    Io(std::io::Error),
}

impl std::error::Error for Error {}

impl From<openapi::apis::Error<ImagePostError>> for Error {
    fn from(value: openapi::apis::Error<ImagePostError>) -> Self {
        Self::Upload(value)
    }
}

impl From<openapi::apis::Error<ImagePutError>> for Error {
    fn from(value: openapi::apis::Error<ImagePutError>) -> Self {
        Self::Create(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
