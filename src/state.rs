use openapi::models::UpdatePutRequest;

use crate::gallery::{GalleryCollectableMediaKind, GalleryItem};

#[derive(Clone, Debug, Default)]
pub enum GlobalState {
    #[default]
    Idle,
    PreparingGalleryPost,
    ReviewingGalleryPost,
    UpdatingKey,
    UpdatingApiEndpoint,
    UpdatingVisionModel,
    PreparingUpdateTranslation,
    ReviewingUpdateTranslation,
}

#[derive(Default, Debug, Clone)]
pub enum PostGalleryState {
    #[default]
    Idle,
    Collecting(Vec<GalleryCollectableMediaKind>),
    AltTextCompensate(Vec<GalleryCollectableMediaKind>),
    Committed(GalleryItem),
}

#[derive(Default, Debug, Clone)]
pub enum UpdateVisionModelState {
    #[default]
    Name,
    Key,
}

#[derive(Default, Debug, Clone)]
pub enum UpdateTranslationState {
    #[default]
    Idle,
    Selected(i32),
    Translated(Vec<UpdatePutRequest>),
}
