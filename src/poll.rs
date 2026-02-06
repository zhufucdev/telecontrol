use std::{collections::HashMap, fmt::Debug, sync::Arc};

use teloxide::{
    dispatching::{DpHandlerDescription, dialogue::Storage},
    dptree::{self, Handler},
    prelude::Dialogue,
    types::{ChatId, Poll, PollId, Update, UpdateKind, UserId},
};
use tokio::sync::Mutex;

pub type ContextByPollId = Arc<Mutex<HashMap<PollId, PollContext>>>;

#[derive(Debug, Clone)]
pub struct PollContext {
    pub original_chat_id: ChatId,
    pub issuer_id: UserId,
}

pub fn new_store() -> ContextByPollId {
    return Arc::new(Mutex::new(HashMap::new()));
}

pub fn enter_dialog<S, D, Output>() -> Handler<'static, Output, DpHandlerDescription>
where
    S: Storage<D> + ?Sized + Send + Sync + 'static,
    <S as Storage<D>>::Error: Debug + Send,
    D: Default + Clone + Send + Sync + 'static,
    Output: Send + Sync + 'static,
{
    dptree::filter_map(|update: Update| {
        let UpdateKind::Poll(poll) = update.kind else {
            return None;
        };
        return Some(poll);
    })
    .filter_map_async(async |store: ContextByPollId, poll: Poll| {
        store.lock().await.get(&poll.id).map(|id| id.clone())
    })
    .filter_map(|storage: Arc<S>, context: PollContext| {
        Some(Dialogue::new(storage, context.original_chat_id))
    })
    .filter_map_async(async |dialogue: Dialogue<D, S>| {
        match dialogue.get_or_default().await {
            Ok(dialogue) => Some(dialogue),
            Err(err) => match std::env::var("TELOXIDE_DIALOGUE_BEHAVIOUR").as_deref() {
                Ok("default") => {
                    let default = D::default();
                    dialogue.update(default.clone()).await.ok()?;
                    Some(default)
                }
                Ok("panic") | Err(_) => {
                    log::error!("dialogue.get_or_default() failed: {err:?}");
                    None
                }
                Ok(_) => {
                    panic!(
                        "`TELOXIDE_DIALOGUE_BEHAVIOUR` env variable should be one of: \
                         default/panic"
                    )
                }
            },
        }
    })
}
