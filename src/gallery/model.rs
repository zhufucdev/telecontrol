use lingua::{Language, LanguageDetectorBuilder};
use openapi::models::SupportedLocale;
use strum::Display;
use teloxide::{
    Bot,
    types::{MediaDocument, MediaPhoto, MediaText},
};

use crate::{
    image::{self, Image},
    locale::{AllCases, FromLanguage, FromSupportedLocale},
};

#[derive(Debug, Clone)]
pub struct GalleryItem {
    pub photo: Image,
    pub tweet: Option<String>,
    pub locale: Option<SupportedLocale>,
}

#[derive(Debug, Clone, Default)]
pub struct ParseMediaConfigurations {
    pub photo_duplication: DuplicationResolutionPolicy,
    pub tweet_duplication: DuplicationResolutionPolicy,
}

impl GalleryItem {
    pub async fn parse_media<I, Media>(
        media: I,
        bot: &Bot,
        config: ParseMediaConfigurations,
    ) -> Result<Self, Error>
    where
        I: IntoIterator<Item = Media>,
        Media: AsRef<GalleryCollectableMediaKind>,
    {
        let media = media.into_iter().collect::<Vec<Media>>();
        let mut alt_text: Option<String> = None;
        let mut locale: Option<SupportedLocale> = None;
        for compensate in media.iter().filter_map(|m| {
            if let GalleryCollectableMediaKind::Compensate(c) = m.as_ref() {
                Some(c)
            } else {
                None
            }
        }) {
            match compensate {
                GalleryCollectableCompensate::AltText(text) => alt_text = Some(text.clone()),
                GalleryCollectableCompensate::Locale(l) => locale = Some(l.clone()),
            }
        }

        match config.photo_duplication {
            DuplicationResolutionPolicy::Concatenate => {
                return Err(Error::IncompatibleDuplicationResolutionPolicy);
            }
            _ => {}
        }
        let mut tweet: Option<String> = None;
        let mut photo: Option<Image> = None;
        for medium in media {
            match medium.as_ref() {
                GalleryCollectableMediaKind::Text(media_text) => {
                    let Some(text) = &tweet else {
                        tweet = Some(media_text.text.clone());
                        continue;
                    };
                    match config.tweet_duplication {
                        DuplicationResolutionPolicy::Error => {
                            return Err(Error::DuplicatedTweet);
                        }
                        DuplicationResolutionPolicy::UseFirst => {}
                        DuplicationResolutionPolicy::UseLast => {
                            tweet = Some(media_text.text.clone())
                        }
                        DuplicationResolutionPolicy::Concatenate => {
                            tweet = Some(format!("{}\n{}", text, media_text.text))
                        }
                    }
                }
                GalleryCollectableMediaKind::Photo(media_photo) => {
                    let Some(_) = &photo else {
                        photo = Some(Image::parse_media_photo(media_photo, bot, &alt_text).await?);
                        continue;
                    };
                    match config.photo_duplication {
                        DuplicationResolutionPolicy::Error => {
                            return Err(Error::DuplicatedPhoto);
                        }
                        DuplicationResolutionPolicy::UseFirst => {}
                        DuplicationResolutionPolicy::UseLast => {
                            photo =
                                Some(Image::parse_media_photo(media_photo, bot, &alt_text).await?)
                        }
                        DuplicationResolutionPolicy::Concatenate => {
                            return Err(Error::IncompatibleDuplicationResolutionPolicy);
                        }
                    }
                }
                GalleryCollectableMediaKind::Document(media_document) => {
                    let Some(_) = &photo else {
                        photo = Some(
                            Image::parse_media_document(media_document, bot, &alt_text).await?,
                        );
                        continue;
                    };
                    match config.photo_duplication {
                        DuplicationResolutionPolicy::Error => {
                            return Err(Error::DuplicatedPhoto);
                        }
                        DuplicationResolutionPolicy::UseFirst => {}
                        DuplicationResolutionPolicy::UseLast => {
                            photo = Some(
                                Image::parse_media_document(media_document, bot, &alt_text)
                                    .await
                                    .map_err(|err| Error::PhotoError(err))?,
                            )
                        }
                        DuplicationResolutionPolicy::Concatenate => {
                            return Err(Error::IncompatibleDuplicationResolutionPolicy);
                        }
                    }
                }
                GalleryCollectableMediaKind::Compensate(_) => {
                    // ignored, already processed before
                }
            }
        }
        if let None = locale
            && let Some(tweet) = &tweet
        {
            let detector = LanguageDetectorBuilder::from_languages(
                SupportedLocale::all_cases()
                    .iter()
                    .map(|locale| Language::from_supported_locale(*locale))
                    .collect::<Vec<Language>>()
                    .as_slice(),
            )
            .build();
            locale = detector
                .detect_language_of(tweet)
                .map(SupportedLocale::from_language);
        }
        Ok(Self {
            photo: photo.ok_or(Error::MissingPhoto)?,
            tweet,
            locale,
        })
    }
}

#[derive(Debug, Clone)]
pub enum GalleryCollectableMediaKind {
    Text(MediaText),
    Photo(MediaPhoto),
    Document(MediaDocument),
    Compensate(GalleryCollectableCompensate),
}

impl AsRef<GalleryCollectableMediaKind> for GalleryCollectableMediaKind {
    fn as_ref(&self) -> &GalleryCollectableMediaKind {
        self
    }
}

#[derive(Debug, Clone)]
pub enum GalleryCollectableCompensate {
    AltText(String),
    Locale(SupportedLocale),
}

#[derive(Debug, Display)]
pub enum Error {
    #[strum(to_string = "missing image")]
    MissingPhoto,
    #[strum(to_string = "incompatible duplication resolution policy")]
    IncompatibleDuplicationResolutionPolicy,
    #[strum(to_string = "duplicated tweet")]
    DuplicatedTweet,
    #[strum(to_string = "duplicated photo")]
    DuplicatedPhoto,
    #[strum(to_string = "photo error: {0}")]
    PhotoError(image::Error),
}

#[derive(Debug, Clone, Default)]
pub enum DuplicationResolutionPolicy {
    #[default]
    Error,
    UseFirst,
    UseLast,
    Concatenate,
}

impl core::error::Error for Error {}

impl From<image::Error> for Error {
    fn from(value: image::Error) -> Self {
        Self::PhotoError(value)
    }
}
