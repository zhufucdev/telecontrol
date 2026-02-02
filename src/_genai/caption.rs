use std::{path::Path, process::Command};

use base64::{Engine, prelude::BASE64_STANDARD};
use genai::chat::{Binary, ChatMessage, ChatRequest, MessageContent};
use strum::Display;
use tokio::fs;

use crate::image::ImageSource;

pub trait GenerateCaption {
    async fn generate_caption<S: AsRef<str>>(
        &self,
        model: S,
        image: ImageSource,
    ) -> Result<String, Error>;
}

impl GenerateCaption for genai::Client {
    async fn generate_caption<S: AsRef<str>>(
        &self,
        model: S,
        image: ImageSource,
    ) -> Result<String, Error> {
        let chat = match image {
            ImageSource::LocalFile(image) => {
                let file_command = Command::new("file")
                    .args(["--mime-type", image.to_str().unwrap()])
                    .output()?;
                let file_output = String::from_utf8(file_command.stdout).unwrap();
                let Some((_, image_mime)) =
                    file_output.strip_suffix("\n").unwrap().split_once(": ")
                else {
                    return Err(Error::UnknownImageType);
                };

                ChatRequest::new(vec![
                    ChatMessage::user(MessageContent::from_text(include_str!(
                        // Credits to https://www.section508.gov/create/alternative-text/
                        "../../prompt/alt_text.md"
                    ))),
                    ChatMessage::user(MessageContent::from(Binary::from_base64(
                        image_mime,
                        BASE64_STANDARD.encode(fs::read(&image).await?),
                        None,
                    ))),
                ])
                .with_system("Describe user's image using at most 2 sentences.")
            }
            ImageSource::Url(photo) => {
                ChatRequest::new(vec![
                    ChatMessage::user(MessageContent::from_text(include_str!(
                        // Credits to https://www.section508.gov/create/alternative-text/
                        "../../prompt/alt_text.md"
                    ))),
                    ChatMessage::user(MessageContent::from(Binary::from_url(
                        "image/jpeg",
                        photo,
                        None,
                    ))), // TODO: replace with actual MIME
                ])
                .with_system("Describe user's image using at most 2 sentences.")
            }
        };
        Ok(self
            .exec_chat(model.as_ref(), chat, None)
            .await?
            .texts()
            .join("\n"))
    }
}

#[derive(Debug, Display)]
pub enum Error {
    #[strum(to_string = "io error: {0}")]
    Io(std::io::Error),
    #[strum(to_string = "unknown image type")]
    UnknownImageType,
    #[strum(to_string = "failed to generate caption: {0}")]
    GenAI(genai::Error),
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<genai::Error> for Error {
    fn from(value: genai::Error) -> Self {
        Self::GenAI(value)
    }
}
