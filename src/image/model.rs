use std::{
    collections::HashMap,
    io,
    path::PathBuf,
    sync::{Arc, LazyLock},
};

use genai::chat::CacheControl;
use strum::Display;
use teloxide::{
    Bot, DownloadError, RequestError,
    net::Download,
    prelude::Requester,
    types::{FileMeta, FileUniqueId, MediaDocument, MediaPhoto},
};
use tempfile::TempDir;
use tokio::{
    fs::{self, File},
    sync::Mutex,
};

use crate::gallery::GalleryCollectableMediaKind;

#[derive(Debug, Clone)]
pub struct Image {
    pub source: ImageSource,
    pub alt_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageSource {
    LocalFile(PathBuf),
    Url(String),
}

static IMAGE_CACHE_REPO: LazyLock<Arc<Mutex<HashMap<FileUniqueId, ImageCacheEntry>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

struct ImageCacheEntry {
    source: ImageSource,
    temp_dir: TempDir,
}

impl ImageSource {
    pub async fn from_file_meta(meta: &FileMeta, bot: &Bot) -> Result<Self, Error> {
        if let Some(cached) = IMAGE_CACHE_REPO.lock().await.get(&meta.unique_id) {
            return Ok(cached.source.clone());
        }
        let remote_file = bot
            .get_file(meta.id.clone())
            .await
            .map_err(|err| Error::MissingMedia(err))?;
        let temp_dir = tempfile::tempdir()?;
        let cache_path = temp_dir
            .path()
            .to_owned()
            .join(remote_file.unique_id.to_string());
        let mut local_file = File::create_new(&cache_path).await?;
        bot.download_file(&remote_file.path, &mut local_file)
            .await?;
        log::info!(
            "cached image source {} to {}",
            meta.unique_id,
            cache_path.display()
        );
        let source = ImageSource::LocalFile(cache_path);
        let mut cache = IMAGE_CACHE_REPO.lock().await;
        cache.insert(
            meta.unique_id.clone(),
            ImageCacheEntry {
                source: source.clone(),
                temp_dir,
            },
        );
        Ok(source)
    }

    pub async fn from_media_photo(media: &MediaPhoto, bot: &Bot) -> Result<Self, Error> {
        let Some(best_photo) = media
            .photo
            .iter()
            .max_by(|a, b| (a.width * a.height).cmp(&(b.width * b.height)))
        else {
            return Err(Error::EmptyMedia);
        };
        ImageSource::from_file_meta(&best_photo.file, bot).await
    }

    pub async fn from_media_document(media: &MediaDocument, bot: &Bot) -> Result<Self, Error> {
        ImageSource::from_file_meta(&media.document.file, bot).await
    }

    pub async fn from_gallery_collectable(
        collectable: &GalleryCollectableMediaKind,
        bot: &Bot,
    ) -> Result<Option<Self>, Error> {
        Ok(match collectable {
            GalleryCollectableMediaKind::Photo(media_photo) => {
                Some(Self::from_media_photo(media_photo, bot).await?)
            }
            GalleryCollectableMediaKind::Document(media_document) => {
                Some(Self::from_media_document(media_document, bot).await?)
            }
            _ => None,
        })
    }

    pub fn filename(&self) -> Option<String> {
        match self {
            ImageSource::LocalFile(path_buf) => path_buf
                .file_name()
                .map(|name| name.to_str())
                .flatten()
                .map(|name| name.to_string()),
            ImageSource::Url(url) => url
                .rsplit_once('/')
                .map(|(l, r)| {
                    if !l.is_empty() {
                        Some(l)
                    } else if !r.is_empty() {
                        Some(r)
                    } else {
                        None
                    }
                })
                .flatten()
                .map(|s| s.to_string()),
        }
    }

    pub async fn dispose(self) -> Result<(), std::io::Error> {
        if let Some((k, _)) = IMAGE_CACHE_REPO
            .lock()
            .await
            .iter()
            .find(|(_, v)| v.source == self)
        {
            let mut cache = IMAGE_CACHE_REPO.lock().await;
            cache.remove(k);
        }
        match &self {
            ImageSource::LocalFile(path_buf) => {
                fs::remove_file(path_buf).await?;
            }
            ImageSource::Url(_) => {}
        }
        Ok(())
    }
}

impl Image {
    pub async fn parse_media_photo(
        media: &MediaPhoto,
        bot: &Bot,
        alt_text: &Option<String>,
    ) -> Result<Self, Error> {
        let caption = media.caption.as_ref();
        let alt_text = alt_text
            .as_ref()
            .or(caption)
            .ok_or(Error::MissingAltText)?
            .clone();

        Ok(Self {
            source: ImageSource::from_media_photo(media, bot).await?,
            alt_text,
        })
    }

    pub async fn parse_media_document(
        media: &MediaDocument,
        bot: &Bot,
        alt_text: &Option<String>,
    ) -> Result<Self, Error> {
        let caption = media.caption.as_ref();
        let alt_text = alt_text
            .as_ref()
            .or(caption)
            .ok_or(Error::MissingAltText)?
            .clone();
        Ok(Self {
            source: ImageSource::from_media_document(media, bot).await?,
            alt_text,
        })
    }
}

#[derive(Debug, Display)]
pub enum Error {
    #[strum(to_string = "missing alt text")]
    MissingAltText,
    #[strum(to_string = "empty media")]
    EmptyMedia,
    #[strum(to_string = "media missing")]
    MissingMedia(RequestError),
    #[strum(to_string = "{0}")]
    IOError(io::Error),
    #[strum(to_string = "{0}")]
    DownloadError(DownloadError),
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::IOError(value)
    }
}

impl From<DownloadError> for Error {
    fn from(value: DownloadError) -> Self {
        Self::DownloadError(value)
    }
}
