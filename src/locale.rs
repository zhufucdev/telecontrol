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
