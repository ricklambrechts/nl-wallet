use std::{collections::HashMap, ops::Index, sync::LazyLock};

use serde::Serialize;

use crate::app::Language;

pub type Translations<'a> = HashMap<Language, Words<'a>>;

pub static TRANSLATIONS: LazyLock<Translations> = LazyLock::new(|| {
    let en = Words {
        en: "English",
        nl: "Nederlands",
        demo_bar_text: "NL Wallet Demo",
        demo_see_other: "View other",
        demo_see_examples: "examples",
        demo_follow_development: "Follow the developments at",
        continue_with_nl_wallet: "Continue with NL Wallet",
        continue_with_google: "Continue with Google",
        continue_with_email: "Continue with email",
        login_with_nl_wallet: "Login with NL Wallet",
        login_with_digid: "Login with DigiD",
        use_nl_wallet: "Use NL Wallet",
        choose_another_method: "Choose another method",
        amsterdam_title: "Municipality of Amsterdam",
        amsterdam_failed: "Login failed",
        amsterdam_try_again: "Try again",
        amsterdam_login: "Login to Mijn Amsterdam",
        amsterdam_subtitle: "For individuals and sole proprietors",
        amsterdam_nl_wallet_digid: "You need the NL Wallet app or DigiD.",
        amsterdam_profile_name: "Account",
        amsterdam_success: "Success",
        amsterdam_logged_in: "You are logged in.",
        amsterdam_welcome: "Welcome to Mijn Amsterdam",
        amsterdam_subtitle_disclosed: "Personal online services for Amsterdam residents",
        monkeybike_title: "MonkeyBike",
        monkeybike_login: "Log in",
        marketplace_title: "Marktplek",
        marketplace_login: "Sign up or log in",
        login_failed_try_again: "Login failed. Try again.",
        click_continue: "By clicking \"Continue\", you agree to the",
        terms_and_conditions: "Terms and Conditions",
        and_the: "and the",
        privacy_policy: "Privacy Policy",
        xyz_title: "XYZ Bank",
        xyz_open_account: "Open bank account",
        xyz_identify_yourself: "Step 1. Identify yourself",
        xyz_failed_try_again: "Identification failed. Try again.",
        xyz_success: "Identification successful",
        welcome: "Welcome",
        search_product: "Search product...",
        search_by_topic: "Search by topic...",
        next: "Next",
    };

    let nl = Words {
        en: "English",
        nl: "Nederlands",
        demo_bar_text: "NL Wallet Demo",
        demo_see_other: "Bekijk andere",
        demo_see_examples: "voorbeelden",
        demo_follow_development: "Volg de ontwikkelingen op",
        continue_with_nl_wallet: "Verder met NL Wallet",
        continue_with_google: "Verder met Google",
        continue_with_email: "Verder met email",
        login_with_nl_wallet: "Inloggen met NL Wallet",
        login_with_digid: "Inloggen met DigiD",
        use_nl_wallet: "Gebruik NL Wallet",
        choose_another_method: "Kies een ander middel",
        amsterdam_title: "Gemeente Amsterdam",
        amsterdam_failed: "Inloggen mislukt",
        amsterdam_try_again: "Probeer het opnieuw",
        amsterdam_login: "Inloggen op Mijn Amsterdam",
        amsterdam_subtitle: "Voor particulieren en eenmanszaken",
        amsterdam_nl_wallet_digid: "U heeft de NL Wallet app of DigiD nodig.",
        amsterdam_profile_name: "Account",
        amsterdam_success: "Gelukt",
        amsterdam_logged_in: "Je bent ingelogd.",
        amsterdam_welcome: "Welkom in Mijn Amsterdam",
        amsterdam_subtitle_disclosed: "Persoonlijke online dienstverlening voor de Amsterdammer",
        monkeybike_title: "MonkeyBike",
        monkeybike_login: "Meld je aan",
        marketplace_title: "Marktplek",
        marketplace_login: "Meld je aan of log in",
        login_failed_try_again: "Inloggen mislukt. Probeer het opnieuw.",
        click_continue: "Door op \"Verder\" te klikken, ga je akkoord met de",
        terms_and_conditions: "Algemene Voorwaarden",
        and_the: "en het",
        privacy_policy: "Privacybeleid",
        xyz_title: "XYZ Bank",
        xyz_open_account: "Bankrekening openen",
        xyz_identify_yourself: "Stap 1. Identificeer uzelf",
        xyz_failed_try_again: "Identificatie mislukt. Probeer het opnieuw.",
        xyz_success: "Identificatie gelukt",
        welcome: "Welkom",
        search_product: "Zoek product...",
        search_by_topic: "Zoek op onderwerp...",
        next: "Volgende",
    };

    let mut translations = HashMap::new();
    translations.insert(Language::Nl, nl);
    translations.insert(Language::En, en);
    translations
});

#[derive(Serialize)]
pub struct Words<'a> {
    en: &'a str,
    nl: &'a str,
    pub demo_bar_text: &'a str,
    pub demo_see_other: &'a str,
    pub demo_see_examples: &'a str,
    pub demo_follow_development: &'a str,
    pub continue_with_nl_wallet: &'a str,
    pub continue_with_google: &'a str,
    pub continue_with_email: &'a str,
    pub login_with_nl_wallet: &'a str,
    pub login_with_digid: &'a str,
    pub use_nl_wallet: &'a str,
    pub choose_another_method: &'a str,
    pub amsterdam_title: &'a str,
    pub amsterdam_failed: &'a str,
    pub amsterdam_try_again: &'a str,
    pub amsterdam_login: &'a str,
    pub amsterdam_subtitle: &'a str,
    pub amsterdam_nl_wallet_digid: &'a str,
    pub amsterdam_profile_name: &'a str,
    pub amsterdam_success: &'a str,
    pub amsterdam_logged_in: &'a str,
    pub amsterdam_welcome: &'a str,
    pub amsterdam_subtitle_disclosed: &'a str,
    pub monkeybike_title: &'a str,
    pub monkeybike_login: &'a str,
    pub marketplace_title: &'a str,
    pub marketplace_login: &'a str,
    pub login_failed_try_again: &'a str,
    pub click_continue: &'a str,
    pub terms_and_conditions: &'a str,
    pub and_the: &'a str,
    pub privacy_policy: &'a str,
    pub xyz_title: &'a str,
    pub xyz_open_account: &'a str,
    pub xyz_identify_yourself: &'a str,
    pub xyz_failed_try_again: &'a str,
    pub xyz_success: &'a str,
    pub welcome: &'a str,
    pub search_product: &'a str,
    pub search_by_topic: &'a str,
    pub next: &'a str,
}

impl<'a> Index<Language> for Words<'a> {
    type Output = &'a str;

    fn index(&self, lang: Language) -> &Self::Output {
        match lang {
            Language::Nl => &self.nl,
            Language::En => &self.en,
        }
    }
}
