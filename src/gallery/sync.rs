use async_stream::try_stream;
use futures::Stream;
use openapi::{
    apis::{
        configuration::Configuration,
        default_api::{self, GalleryPutError},
    },
    models::GalleryPutRequest,
};
use strum::Display;

use crate::{
    gallery::GalleryItem,
    image::{self},
};

impl GalleryItem {
    pub fn push(&self, configuration: &Configuration) -> impl Stream<Item = Result<State, Error>> {
        try_stream! {
            yield State::UploadingImage;
            let remote_photo = self.photo.push(configuration).await?;
            yield State::CreatingPost;
            let req = GalleryPutRequest::new(self.locale, self.tweet.clone(), remote_photo);
            let gallery_id = default_api::gallery_put(configuration, req).await?;
            yield State::Completed(gallery_id);
        }
    }
}

#[derive(Debug)]
pub enum State {
    UploadingImage,
    CreatingPost,
    Completed(i32),
}

#[derive(Debug, Display)]
pub enum Error {
    #[strum(to_string = "push photo failed to {0}")]
    PushPhoto(image::sync::Error),
    #[strum(to_string = "post creation failed: {0}")]
    CreatePost(openapi::apis::Error<GalleryPutError>),
}

impl std::error::Error for Error {}

impl From<image::sync::Error> for Error {
    fn from(value: image::sync::Error) -> Self {
        Self::PushPhoto(value)
    }
}

impl From<openapi::apis::Error<GalleryPutError>> for Error {
    fn from(value: openapi::apis::Error<GalleryPutError>) -> Self {
        Self::CreatePost(value)
    }
}
