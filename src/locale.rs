use openapi::models::SupportedLocale;

pub trait FromSupportedLocale {
    fn from_supported_locale(locale: SupportedLocale) -> Self;
}

pub trait FromLanguage {
    fn from_language(language: lingua::Language) -> Self;
}

pub trait AllCases {
    fn all_cases() -> &'static [Self]
    where
        Self: Sized;
}

impl FromSupportedLocale for lingua::Language {
    fn from_supported_locale(locale: SupportedLocale) -> Self {
        match locale {
            SupportedLocale::En => lingua::Language::English,
            SupportedLocale::Zh => lingua::Language::Chinese,
            // None of the libraries distinguish
            // between Chinese and Taiwanese, unfortunately
            SupportedLocale::ZhTw => lingua::Language::Chinese,
        }
    }
}

impl AllCases for SupportedLocale {
    fn all_cases() -> &'static [Self]
    where
        Self: Sized,
    {
        return &[
            SupportedLocale::En,
            SupportedLocale::ZhTw,
            SupportedLocale::Zh,
        ];
    }
}

impl FromLanguage for SupportedLocale {
    fn from_language(language: lingua::Language) -> Self {
        match language {
            lingua::Language::Chinese => SupportedLocale::Zh,
            lingua::Language::English => SupportedLocale::ZhTw,
        }
    }
}

pub trait LocaleLanguageName {
    fn typical_language_name(&self) -> &'static str;
    fn from_typical_language_name(name: &str) -> Option<Self>
    where
        Self: Sized;
}

impl LocaleLanguageName for SupportedLocale {
    fn typical_language_name(&self) -> &'static str {
        match self {
            SupportedLocale::En => "English",
            SupportedLocale::Zh => "Simplified Chinese",
            SupportedLocale::ZhTw => "Traditional Chinese",
        }
    }

    fn from_typical_language_name(name: &str) -> Option<Self> {
        match name {
            "English" => Some(SupportedLocale::En),
            "Simplified Chinese" => Some(SupportedLocale::Zh),
            "Traditional Chinese" => Some(SupportedLocale::ZhTw),
            _ => None,
        }
    }
}
