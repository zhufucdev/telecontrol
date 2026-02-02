use genai::adapter::AdapterKind;

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
