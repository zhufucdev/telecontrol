use aes_gcm_siv::aead::{OsRng, rand_core::RngCore};
use futures::future;
use genai::chat::{ChatMessage, ChatRequest};
use openapi::{
    apis::default_api,
    models::{SupportedLocale, UpdatePost, UpdatePutRequest},
};
use serde::Deserialize;
use strum::Display;

use crate::locale::LocaleLanguageName;

pub trait TranslateTo<Output, Err> {
    async fn translate_to<M, R>(
        &self,
        locale: SupportedLocale,
        client: &genai::Client,
        model: M,
        api_config: &openapi::apis::configuration::Configuration,
        reference_translations: Option<R>,
    ) -> Result<Option<Output>, Err>
    where
        M: AsRef<str>,
        R: AsRef<[Output]>,
        Output: Sized;
}

fn get_random_string() -> String {
    let mut key = [0u8; 8];
    OsRng.fill_bytes(&mut key);
    let dictionary = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789)_";
    key.iter()
        .map(|&b| char::from(dictionary[(b as usize) % dictionary.len()]))
        .collect()
}

impl TranslateTo<UpdatePutRequest, Error> for UpdatePost {
    async fn translate_to<M, R>(
        &self,
        locale: SupportedLocale,
        client: &genai::Client,
        model: M,
        api_config: &openapi::apis::configuration::Configuration,
        context_window: Option<R>,
    ) -> Result<Option<UpdatePutRequest>, Error>
    where
        M: AsRef<str>,
        R: AsRef<[UpdatePutRequest]>,
        Option<UpdatePutRequest>: Sized,
    {
        if locale == self.locale {
            return Ok(None);
        }

        #[derive(Deserialize)]
        struct Root {
            title: String,
            summary: String,
        }
        let prompt_boundary_system_prompt = include_str!("../../prompt/prompt_boundary.md");
        let gen_lm = async || -> Result<Root, Error> {
            let request = ChatRequest::new(
                context_window
                    .as_ref()
                    .map(|context_window| {
                        context_window
                            .as_ref()
                            .iter()
                            .map(|context| {
                                [
                                    ChatMessage::user(translate_title_summary_request_template(
                                        self.locale,
                                        context.locale,
                                        &self.title,
                                        &self.summary,
                                    )),
                                    ChatMessage::assistant(translate_title_summary_response(
                                        &context.title,
                                        &context.summary,
                                    )),
                                ]
                            })
                            .flatten()
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default()
                    .into_iter()
                    .chain([ChatMessage::user(translate_title_summary_request_template(
                        self.locale,
                        locale,
                        &self.title,
                        &self.summary,
                    ))])
                    .collect(),
            )
            .with_system(prompt_boundary_system_prompt);
            log::debug!(target: "genai", "title summary request: {:?}", request.messages);

            let lm_output = client
                .exec_chat(model.as_ref(), request, None)
                .await?
                .texts()
                .join("\n");
            log::debug!(target: "genai", "title summary response: {lm_output}");
            let lm: Root = quick_xml::de::from_str(&lm_output)?;
            Ok(lm)
        };
        let gen_header = async || -> Result<String, Error> {
            let locale_name = locale.to_string();
            let header_options =
                default_api::strings_by_locale_locale_get(api_config, &locale_name).await;
            let request = ChatRequest::new(
                context_window
                    .as_ref()
                    .map(|context_window| {
                        context_window
                            .as_ref()
                            .iter()
                            .map(|context| {
                                [
                                    ChatMessage::user(translate_header_request_template(
                                        self.locale,
                                        context.locale,
                                        &context.header,
                                    )),
                                    ChatMessage::assistant(translate_header_response(
                                        &context.header,
                                    )),
                                ]
                            })
                            .flatten()
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default()
                    .into_iter()
                    .chain([ChatMessage::user(match header_options {
                        Ok(header_options) => translate_header_request_template_with_dic(
                            self.locale,
                            locale,
                            &self.header,
                            header_options,
                        ),
                        Err(err) => {
                            log::error!("failed to get string list: {err}");

                            translate_header_request_template(self.locale, locale, &self.header)
                        }
                    })])
                    .collect(),
            )
            .with_system(prompt_boundary_system_prompt);
            log::debug!(target: "genai", "header request: {:?}", request.messages);
            let header = client
                .exec_chat(model.as_ref(), request, None)
                .await?
                .texts()
                .join("\n");
            log::debug!(target: "genai", "header response: {header}");
            Ok(header)
        };
        let (lm, header) = future::try_join(gen_lm(), gen_header()).await?;

        Ok(Some(UpdatePutRequest {
            locale,
            header,
            title: lm.title,
            summary: lm.summary,
            cover: self.cover.clone().map(|c| c.id),
            mask: self.mask,
        }))
    }
}

#[derive(Debug, Display)]
pub enum Error {
    #[strum(to_string = "{0}")]
    GenAI(genai::Error),
    #[strum(to_string = "XML parse error: {0}")]
    InvalidXml(quick_xml::de::DeError),
}

impl std::error::Error for Error {}

impl From<genai::Error> for Error {
    fn from(value: genai::Error) -> Self {
        Self::GenAI(value)
    }
}

impl From<quick_xml::de::DeError> for Error {
    fn from(value: quick_xml::de::DeError) -> Self {
        Self::InvalidXml(value)
    }
}

fn translate_title_summary_request_template(
    source_locale: SupportedLocale,
    target_locale: SupportedLocale,
    title: &str,
    summary: &str,
) -> String {
    let (b_1, b_2) = (get_random_string(), get_random_string());
    format!("You are tasked with translating the following social media tweet from {0} to {1}, wrapped in prompt boundary {b_1} and {b_2}
Output the translated data in XML. Do not include the prompt boundary wrapper.
<tweet>
<title>--prompt boundary {b_1}
{2}
--prompt boundary {b_1}--</title>
<summary>--prompt boundary {b_2}
{3}
--prompt boundary {b_2}--</summary>
</tweet>", source_locale.typical_language_name(), target_locale.typical_language_name(), title, summary)
}

fn translate_title_summary_response<S: AsRef<str>>(title: S, summary: S) -> String {
    format!(
        "<tweet>
<title>{0}</title>
<summary>{1}</summary>
</tweet>",
        title.as_ref(),
        summary.as_ref()
    )
}

fn translate_header_request_template_with_dic<S, V>(
    source_locale: SupportedLocale,
    target_locale: SupportedLocale,
    header: S,
    dictionary: V,
) -> String
where
    S: AsRef<str>,
    V: AsRef<[String]>,
{
    let (b_1, b_2) = (get_random_string(), get_random_string());
    format!("You are tasked with translating the following text from {0} to {1}. Choose from this dictionary if possible.
<text>
--prompt boundary {b_1}
{2}
--prompt boundary {b_1}--
</text>

<dictionary>
--prompt boundary {b_2}
{3}
--prompt boundary {b_2}--
</dictionary>

If there's no viable translation, use a custom one. Keep the translation short and clean. Either case, output in plain text. Do not include the prompt boundary wrapper nor XML tag.", source_locale.typical_language_name(), target_locale.typical_language_name(), header.as_ref(), dictionary.as_ref().join("\n\n"))
}

fn translate_header_request_template<S>(
    source_locale: SupportedLocale,
    target_locale: SupportedLocale,
    header: S,
) -> String
where
    S: AsRef<str>,
{
    let b_1 = get_random_string();
    format!(
        "You are tasked with translating the following text from {0} to {1}.
---prompt boundary {b_1}
{2}
---prompt boundary {b_1}---

Output in plain text. Do not include the prompt boundary.",
        source_locale.typical_language_name(),
        target_locale.typical_language_name(),
        header.as_ref()
    )
}

fn translate_header_response<S: AsRef<str>>(header: S) -> String {
    format!("{}", header.as_ref())
}
