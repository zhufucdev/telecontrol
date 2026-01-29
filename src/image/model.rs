use std::{io, path::PathBuf};

use strum::Display;
use teloxide::{
    Bot, DownloadError, RequestError,
    net::Download,
    prelude::Requester,
    types::{FileMeta, MediaDocument, MediaPhoto},
};
use tokio::fs::{self, File};

#[derive(Debug, Clone)]
pub struct Image {
    pub source: ImageSource,
    pub alt_text: String,
}

#[derive(Debug, Clone)]
pub enum ImageSource {
    LocalFile(PathBuf),
    Url(String),
}

impl ImageSource {
    pub async fn from_file_meta(meta: &FileMeta, bot: &Bot) -> Result<Self, Error> {
        let remote_file = bot
            .get_file(meta.id.clone())
            .await
            .map_err(|err| Error::MissingMedia(err))?;
        let cache_path =
            tempfile::tempdir().map(|dir| dir.path().join(remote_file.unique_id.to_string()))?;
        if let Some(cache) = cache_path.parent() {
            log::info!("Creating cache directory {}", cache.display());
            if !cache.exists() {
                fs::create_dir_all(cache).await?;
            }
        }
        let mut local_file = File::create_new(&cache_path).await?;
        bot.download_file(&remote_file.path, &mut local_file)
            .await?;
        log::info!(
            "Cached image source {} to {}",
            meta.unique_id,
            cache_path.display()
        );
        Ok(ImageSource::LocalFile(cache_path))
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
