use std::{env, net::IpAddr, path::PathBuf};

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use serde_with::{base64::Base64, serde_as};

use wallet_common::{config::wallet_config::BaseUrl, reqwest::deserialize_certificate};

#[derive(Clone, Deserialize)]
pub struct Settings {
    pub ip: IpAddr,
    pub port: u16,

    pub gbav: GbavSettings,

    pub preloaded_xml_path: Option<String>,
}

#[serde_as]
#[derive(Clone, Deserialize)]
pub struct GbavSettings {
    pub adhoc_url: BaseUrl,
    pub username: String,
    pub password: String,

    #[serde_as(as = "Base64")]
    pub client_cert: Vec<u8>,

    #[serde_as(as = "Base64")]
    pub client_cert_key: Vec<u8>,

    #[serde(deserialize_with = "deserialize_certificate")]
    pub trust_anchor: reqwest::Certificate,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        // Look for a config file that is in the same directory as Cargo.toml if run through cargo,
        // otherwise look in the current working directory.
        let config_path = env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap_or_default();

        Config::builder()
            .set_default("ip", "0.0.0.0")?
            .set_default("port", 3006)?
            .add_source(File::from(config_path.join("gba-hc-converter.toml")).required(false))
            .add_source(
                Environment::with_prefix("gba_hc_converter")
                    .separator("__")
                    .prefix_separator("_")
                    .list_separator("|"),
            )
            .build()?
            .try_deserialize()
    }
}
