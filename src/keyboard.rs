use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

pub fn inline_good_bad_buttons() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([[
        InlineKeyboardButton::callback("Good", "Good"),
        InlineKeyboardButton::callback("Bad", "Bad"),
    ]])
}
