use std::{
    env,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use clap::Parser;
use futures::{TryStreamExt, pin_mut};
use regex::Regex;
use strum::{Display, EnumIter, EnumString, IntoEnumIterator};
use teloxide::{
    dispatching::{
        UpdateHandler,
        dialogue::{GetChatId, InMemStorage},
    },
    macros::BotCommands,
    payloads::SendMessageSetters,
    prelude::*,
    types::{
        InlineKeyboardButton, InlineKeyboardMarkup, KeyboardButton, KeyboardMarkup, MediaKind,
        MessageKind, ReplyMarkup,
    },
    utils::command::BotCommands as _,
};
use tokio::fs::{self};

use crate::{
    cli::{Cli, DEFAULT_DATABASE_PATH},
    gallery::{GalleryCollectableMediaKind, GalleryItem, ParseMediaConfigurations},
    image::ImageSource,
    kvstore::{KVStore, aes::AesEncryptedKV, heed::HeedKV, structured::SerdeKV},
    privkey::PrivateKey,
};

mod cli;
mod gallery;
mod image;
mod keyboard;
mod kvstore;
mod locale;
mod privkey;
mod user;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let cli = Cli::parse();
    match cli.command {
        Some(cli::Command::Privkey) => {
            println!("{}", PrivateKey::new().to_string())
        }
        Some(cli::Command::Start {
            privkey,
            token,
            database,
        }) => {
            log::info!("Starting telecontrol...");
            let telegram_token = token.or(dotenv::var("BOT_TOKEN").ok()).unwrap_or_else(|| {
                env::var("TC_BOT_TOKEN").expect("Enviroment variable TC_BOT_TOKEN")
            });
            let private_key = privkey
                .or(dotenv::var("PRIVATE_KEY").ok())
                .unwrap_or_else(|| {
                    env::var("TC_PRIVATE_KEY").expect("Enviroment variable TC_PRIVATE_KEY")
                });
            bot_loop(&telegram_token, &private_key, &database)
                .await
                .expect("Bot init");
        }
        None => {
            log::info!("Starting telecontrol...");
            let telegram_token = dotenv::var("BOT_TOKEN")
                .or(env::var("TC_BOT_TOKEN"))
                .expect("Enviroment variable TC_BOT_TOKEN");
            let private_key = dotenv::var("PRIVATE_KEY")
                .or(env::var("TC_PRIVATE_KEY"))
                .expect("Enviroment variable TC_PRIVATE_KEY");
            let database = PathBuf::from_str(DEFAULT_DATABASE_PATH).unwrap();
            bot_loop(&telegram_token, &private_key, &database)
                .await
                .expect("Bot init");
        }
    }
}

async fn bot_loop(
    telegram_token: &str,
    private_key: &str,
    database: &Path,
) -> Result<(), anyhow::Error> {
    if !database.exists() {
        fs::create_dir_all(database).await?;
    }

    let bot = Bot::new(telegram_token);
    let kv = Arc::new(SerdeKV::new(AesEncryptedKV::new(
        HeedKV::new(database, "user_config")?,
        &PrivateKey::from_string(private_key)?,
    )));
    Dispatcher::builder(bot, bot_state_machine())
        .dependencies(dptree::deps![
            InMemStorage::<GlobalState>::new(),
            InMemStorage::<PostGalleryState>::new(),
            kv
        ])
        .enable_ctrlc_handler()
        .error_handler(Arc::new(error_handler))
        .build()
        .dispatch()
        .await;
    Ok(())
}

fn bot_state_machine() -> UpdateHandler<anyhow::Error> {
    dptree::entry()
        .branch(
            Update::filter_message()
                .enter_dialogue::<Message, InMemStorage<GlobalState>, GlobalState>()
                .branch(dptree::case![GlobalState::UpdatingKey].endpoint(handle_update_key))
                .branch(dptree::case![GlobalState::UpdatingApiEndpoint].endpoint(handle_update_api_endpoint))
                .branch(
                    dptree::case![GlobalState::Idle]
                        .filter_command::<Command>()
                        .branch(dptree::case![Command::Post].endpoint(handle_post_command))
                        .branch(dptree::case![Command::SetKey].endpoint(handle_set_key_command))
                        .branch(dptree::case![Command::SetApi].endpoint(handle_set_api_command))
                        .branch(dptree::case![Command::Help].endpoint(handle_help_command)),
                )
                .branch(
                    dptree::case![GlobalState::PreparingGalleryPost]
                        .enter_dialogue::<Message, InMemStorage<PostGalleryState>, PostGalleryState>()
                        .endpoint(handle_prepare_gallery_post),
                )
        )
        .branch(
            Update::filter_callback_query()
                .enter_dialogue::<CallbackQuery, InMemStorage<GlobalState>, GlobalState>()
                .branch(dptree::case![GlobalState::Idle].endpoint(handle_post_callback))
                .branch(
                    dptree::case![GlobalState::ReviewingGalleryPost]
                        .enter_dialogue::<CallbackQuery, InMemStorage<PostGalleryState>, PostGalleryState>()
                        .endpoint(handle_review_gallery_post)
                ),
        )
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "Available Telecontrol commands:"
)]
enum Command {
    #[command(description = "display this menu")]
    Help,
    #[command(description = "update your post authorization key")]
    SetKey,
    #[command(description = "update your API endpoint, only http(s) are supported")]
    SetApi,
    #[command(description = "I want to post stuff")]
    Post,
}

#[derive(Clone, Debug, Default)]
enum GlobalState {
    #[default]
    Idle,
    PreparingGalleryPost,
    ReviewingGalleryPost,
    UpdatingKey,
    UpdatingApiEndpoint,
}

type GlobalDialog = Dialogue<GlobalState, InMemStorage<GlobalState>>;
type PostGalleryDialog = Dialogue<PostGalleryState, InMemStorage<PostGalleryState>>;
type UserConfigKV = SerdeKV<AesEncryptedKV<HeedKV>>;

async fn handle_help_command(bot: Bot, message: Message) -> anyhow::Result<()> {
    bot.send_message(message.chat.id, Command::descriptions().to_string())
        .await?;
    Ok(())
}

async fn handle_post_command(
    bot: Bot,
    message: Message,
    keys: Arc<UserConfigKV>,
) -> anyhow::Result<()> {
    let Some(user) = message.from else {
        return Ok(());
    };
    if !keys.contains(user.id)? {
        bot.send_message(message.chat.id, "You should /setkey first")
            .await?;
        return Ok(());
    }
    let actions =
        InlineKeyboardMarkup::new(PostKind::iter().collect::<Vec<PostKind>>().chunks(3).map(
            |row| {
                row.into_iter()
                    .map(|name| InlineKeyboardButton::callback(name.to_string(), name.to_string()))
            },
        ));

    bot.send_message(message.chat.id, "Choose what you want!")
        .reply_markup(actions)
        .await?;
    Ok(())
}

#[derive(Debug, Clone, Copy, EnumString, EnumIter, Display)]
enum PostKind {
    Update,
    Gallery,
}

async fn handle_set_key_command(
    bot: Bot,
    message: Message,
    dialog: GlobalDialog,
) -> anyhow::Result<()> {
    dialog.update(GlobalState::UpdatingKey).await?;
    bot.send_message(
        message.chat.id,
        "Send new key to update, after which sensitive data will be erased.",
    )
    .await?;
    Ok(())
}

async fn handle_set_api_command(
    bot: Bot,
    message: Message,
    dialog: GlobalDialog,
) -> anyhow::Result<()> {
    dialog.update(GlobalState::UpdatingApiEndpoint).await?;
    bot.send_message(
        message.chat.id,
        "Send new API endpoint, beginning with http:// or https://",
    )
    .await?;
    Ok(())
}

async fn handle_update_key(
    bot: Bot,
    message: Message,
    dialog: GlobalDialog,
    keys: Arc<UserConfigKV>,
) -> anyhow::Result<()> {
    async fn inner(keys: &UserConfigKV, key: &str, user: UserId) -> anyhow::Result<()> {
        let config = keys.get(user)?.unwrap_or_default();
        let new_config = user::Configuration {
            post_auth_key: Some(key.to_string()),
            ..config
        };

        keys.set(user, &new_config)?;
        Ok(())
    }

    let Some(user) = &message.from else {
        return Ok(());
    };
    let Some(key) = message.text() else {
        bot.send_message(message.chat.id, "You should send a text message")
            .await?;
        return Ok(());
    };

    match inner(&keys, key, user.id).await {
        Ok(_) => {
            bot.send_message(message.chat.id, "Post authorization key has been updated.")
                .await?;
            dialog.update(GlobalState::Idle).await?;
        }
        Err(err) => {
            log::error!("Failed to update user key: {err}");
            bot.send_message(
                message.chat.id,
                "Failed to update key. Please refer to console logs for detail.",
            )
            .await?;
        }
    }
    bot.delete_message(message.chat.id, message.id).await?;
    Ok(())
}

async fn handle_update_api_endpoint(
    bot: Bot,
    message: Message,
    kv: Arc<UserConfigKV>,
    dialog: GlobalDialog,
) -> anyhow::Result<()> {
    async fn inner(user: UserId, endpoint: &str, kv: &UserConfigKV) -> anyhow::Result<()> {
        let config: user::Configuration = kv.get(user)?.unwrap_or_default();
        let new_config = user::Configuration {
            endpoint: Some(endpoint.to_string()),
            ..config
        };
        kv.set(user, &new_config)?;
        Ok(())
    }

    let Some(user) = &message.from else {
        return Ok(());
    };
    let Some(endpoint) = message.text() else {
        bot.send_message(message.chat.id, "You should send a text message")
            .await?;
        return Ok(());
    };
    if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") {
        bot.send_message(
            message.chat.id,
            "Whatever you've sent is not an HTTP(s) url and is unsupported. Try again?",
        )
        .await?;
        return Ok(());
    }
    match inner(user.id, endpoint, &kv).await {
        Ok(_) => {
            bot.send_message(message.chat.id, "API endpoint base URL has been updated.")
                .await?;
            dialog.update(GlobalState::Idle).await?;
        }
        Err(err) => {
            log::error!("Failed to update API endpoint: {err}");
            bot.send_message(
                message.chat.id,
                "Failed to update API endpoint. Please refer to console logs for detail.",
            )
            .await?;
        }
    }
    Ok(())
}

#[derive(Default, Debug, Clone)]
enum PostGalleryState {
    #[default]
    Idle,
    Collecting(Vec<GalleryCollectableMediaKind>),
    AltTextCompensate(Vec<GalleryCollectableMediaKind>),
    Committed(GalleryItem),
}

async fn handle_post_callback(
    bot: Bot,
    query: CallbackQuery,
    global: GlobalDialog,
) -> anyhow::Result<()> {
    let Some(name) = &query.data else {
        return Ok(());
    };
    let Ok(post_kind) = PostKind::from_str(name) else {
        return Ok(());
    };
    match post_kind {
        PostKind::Update => todo!(),
        PostKind::Gallery => {
            global.update(GlobalState::PreparingGalleryPost).await?;
            let keyboard = KeyboardMarkup::new(
                [[KeyboardButton::new("Done"), KeyboardButton::new("Cancel")]].into_iter(),
            );
            bot.send_message(
                        query.chat_id().expect("ChatID unavailable"),
                        "Send me the photo you would like to post! Leave a funny comment if you feel like so~",
                    )
                        .reply_markup(keyboard)
                    .await?;
        }
    }
    bot.answer_callback_query(query.id).await?;
    Ok(())
}

async fn handle_prepare_gallery_post(
    bot: Bot,
    message: Message,
    dialog: PostGalleryDialog,
    global: GlobalDialog,
) -> anyhow::Result<()> {
    async fn send_unsupported_alert(bot: Bot, chat_id: ChatId) -> anyhow::Result<()> {
        bot.send_message(
            chat_id,
            "You just sent something unsupported. I will ignore it for now.",
        )
        .await?;
        Ok(())
    }
    async fn append_to_collecting_state(
        bot: Bot,
        chat_id: ChatId,
        message: GalleryCollectableMediaKind,
        dialog: PostGalleryDialog,
    ) -> anyhow::Result<()> {
        let current = dialog.get_or_default().await?;
        match current {
            PostGalleryState::Idle => {
                dialog
                    .update(PostGalleryState::Collecting(vec![message]))
                    .await?;
            }
            PostGalleryState::Collecting(mut messages) => {
                messages.push(message);
                dialog
                    .update(PostGalleryState::Collecting(messages))
                    .await?;
            }
            PostGalleryState::AltTextCompensate(mut messages) => {
                let GalleryCollectableMediaKind::Text(message) = message else {
                    bot.send_message(chat_id, "Only text message is supported. Try again :)")
                        .await?;
                    return Ok(());
                };
                messages.push(GalleryCollectableMediaKind::Compensate(
                    gallery::GalleryCollectableCompensate::AltText(message.text),
                ));
                dialog
                    .update(PostGalleryState::Collecting(messages))
                    .await?;
                bot.send_message(chat_id, "Gotcha").await?;
            }
            PostGalleryState::Committed(_) => {
                panic!("Illegal state: Committed");
            }
        }
        Ok(())
    }
    let chat_id = message.chat.id;
    let MessageKind::Common(common_msg) = message.kind else {
        send_unsupported_alert(bot, chat_id).await?;
        return Ok(());
    };
    match common_msg.media_kind {
        MediaKind::Text(text) => {
            match text.text.as_str() {
                "Done" => {
                    let Some(PostGalleryState::Collecting(media)) = dialog.get().await? else {
                        bot.send_message(chat_id, "You have yet sent no material to share. Wanna cancel? Press the button!").await?;
                        return Ok(());
                    };
                    match gallery::GalleryItem::parse_media(
                        &media,
                        &bot,
                        ParseMediaConfigurations {
                            ..Default::default()
                        },
                    )
                    .await
                    {
                        Ok(item) => {
                            bot.send_message(chat_id, format!("Let's review beforehand.\nYou have an image, whose alt text is:\n{}", item.photo.alt_text))
                                .reply_markup(ReplyMarkup::kb_remove())
                                .await?;
                            if let Some(tweet) = &item.tweet {
                                bot.send_message(
                                    chat_id,
                                    format!("You also tweeted about it:\n{}", tweet),
                                )
                                .await?;
                            }
                            dialog.update(PostGalleryState::Committed(item)).await?;
                            global.update(GlobalState::ReviewingGalleryPost).await?;
                            bot.send_message(chat_id, "How does this look?")
                                .reply_markup(keyboard::inline_good_bad_buttons())
                                .await?;
                        }
                        Err(err) => match err {
                            gallery::Error::PhotoError(image::Error::MissingAltText) => {
                                bot.send_message(chat_id, "The photo doesn't have a caption. Send me one! Describe the content of the image in brief.").await?;
                                dialog
                                    .update(PostGalleryState::AltTextCompensate(media))
                                    .await?;
                            }
                            _ => {
                                bot.send_message(chat_id, format!("From the dialog, we have {err}. With that in mind, try again!")).await?;
                            }
                        },
                    }
                }
                "Cancel" => {
                    dialog.exit().await?;
                    global.reset().await?;
                    bot.send_message(chat_id, "OK. You may come back anytime.")
                        .reply_markup(ReplyMarkup::kb_remove())
                        .await?;
                }

                _ => {
                    append_to_collecting_state(
                        bot,
                        chat_id,
                        GalleryCollectableMediaKind::Text(text),
                        dialog,
                    )
                    .await?;
                }
            }
        }
        MediaKind::Photo(photo) => {
            append_to_collecting_state(
                bot,
                chat_id,
                GalleryCollectableMediaKind::Photo(photo),
                dialog,
            )
            .await?;
        }
        MediaKind::Document(document) => {
            append_to_collecting_state(
                bot,
                chat_id,
                GalleryCollectableMediaKind::Document(document),
                dialog,
            )
            .await?;
        }
        _ => {
            send_unsupported_alert(bot, chat_id).await?;
        }
    }
    Ok(())
}

async fn handle_review_gallery_post(
    bot: Bot,
    callback: CallbackQuery,
    dialog: PostGalleryDialog,
    global: GlobalDialog,
    kv: Arc<UserConfigKV>,
) -> anyhow::Result<()> {
    let chat_id = callback.chat_id().unwrap();
    let Some(data) = &callback.data else {
        log::error!("Empty gallery post reivision callback data");
        return Ok(());
    };
    let Some(PostGalleryState::Committed(post)) = dialog.get().await? else {
        bot.send_message(
            chat_id,
            "You don't have anything to review yet! Are you using a hacked client or something?",
        )
        .await?;
        bot.answer_callback_query(callback.id).await?;
        return Ok(());
    };
    match data.as_str() {
        "Good" => {
            bot.send_message(chat_id, "Perfect! I will create the post for ya")
                .await?;
            bot.answer_callback_query(callback.id).await?;
            let user_config = kv
                .get::<user::Configuration>(callback.from.id)?
                .unwrap_or_default();
            let endpoint = user_config.endpoint.clone();
            let api_config = &user_config.to_openapi();
            let states = post.push(api_config);
            let mut gallery_id = -1;
            pin_mut!(states);
            loop {
                match states.try_next().await {
                    Err(err) => {
                        log::error!("Failed to push to gallery: {err}");
                        bot.send_message(
                            chat_id,
                            format!(
                                "Oops, I encountered an error where {}. Press the buttons to retry? Better luck next time!",
                                err
                            ),
                        )
                        .reply_markup(keyboard::inline_good_bad_buttons())
                        .await?;
                        return Ok(());
                    }
                    Ok(None) => {
                        break;
                    }
                    Ok(Some(state)) => match state {
                        gallery::sync::State::UploadingImage => {
                            bot.send_message(chat_id, "Uploading photo...")
                                .reply_markup(ReplyMarkup::kb_remove())
                                .await?;
                        }
                        gallery::sync::State::CreatingPost => {
                            bot.send_message(chat_id, "Creating content...")
                                .reply_markup(ReplyMarkup::kb_remove())
                                .await?;
                        }
                        gallery::sync::State::Completed(id) => {
                            gallery_id = id;
                        }
                    },
                }
            }
            let standard_base_site = Regex::new(r"(https?:\/\/[\w.]+(?::\d+)?)\/api\/?").unwrap();
            if let Some(base_site_url) = standard_base_site
                .captures(&endpoint.unwrap_or(user::defaults::DEFAULT_API_ENDPOINT.to_string()))
                && gallery_id >= 0
            {
                _ = bot
                    .send_message(
                        chat_id,
                        format!(
                            "The post has been created~ Check it out! {}/gallery/{}",
                            base_site_url.get(1).unwrap().as_str(),
                            gallery_id
                        ),
                    )
                    .await;
            } else {
                _ = bot
                    .send_message(chat_id, "The post has been created~")
                    .await;
            }
        }
        "Bad" => {
            bot.send_message(chat_id, "No pressures! I will forget what you have posted. Tell me if you have anything new!").await?;
            bot.answer_callback_query(callback.id).await?;
        }
        _ => {
            log::error!("Unknown gallery post reivision callback data {data}");
            return Ok(());
        }
    }
    dialog.exit().await?;
    global.reset().await?;
    if let ImageSource::LocalFile(file) = post.photo.source {
        fs::remove_file(file).await?;
    }
    Ok(())
}

async fn error_handler(error: anyhow::Error) {
    log::error!("{error}")
}
