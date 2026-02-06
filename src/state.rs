use openapi::models::{SupportedLocale, UpdatePost, UpdatePutRequest};

use crate::gallery::{GalleryCollectableMediaKind, GalleryItem};

#[derive(Clone, Debug, Default)]
pub enum GlobalState {
    #[default]
    Idle,
    PostRequestd,
    PreparingGalleryPost,
    ReviewingGalleryPost,
    UpdatingKey,
    UpdatingApiEndpoint,
    UpdatingVisionModel,
    TranslationRequested,
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
    Selected(UpdatePost),
    Translated {
        selected_post: UpdatePost,
        posts: Vec<UpdatePutRequest>,
        failed: Vec<SupportedLocale>,
    },
}
