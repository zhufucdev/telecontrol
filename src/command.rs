use teloxide::macros::BotCommands;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "Available Telecontrol commands:"
)]
pub enum Command {
    #[command(description = "display this menu")]
    Help,
    #[command(description = "update your post authorization key")]
    SetKey,
    #[command(description = "update your API endpoint, only http(s) are supported")]
    SetApi,
    #[command(
        description = "update your vision model name and key, enabling VLM based caption generation"
    )]
    SetVisionModel,
    #[command(description = "I want to post stuff")]
    Post,
    #[command(description = "I want to translate stuff")]
    Translate,
}
