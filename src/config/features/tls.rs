use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TLS {
    BearSSL,
    WolfSSL,
    OpenSSL,
    GnuTLS,
    MbedTLS,
    RusTLS,
}

impl FromStr for TLS {
    type Err = serde_yaml::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let result = match value {
            "bearssl" => Self::BearSSL,
            "wolfssl" => Self::WolfSSL,
            "openssl" => Self::OpenSSL,
            "gnutls" => Self::GnuTLS,
            "mbedtls" => Self::MbedTLS,
            "rustls" => Self::RusTLS,
            _ => bail_de!("Invalid tls type: {}", value),
        };
        Ok(result)
    }
}

impl TLS {
    pub(super) fn to_image_install_value(&self) -> String {
        match self {
            Self::BearSSL => "bearssl",
            Self::WolfSSL => "wolfssl",
            Self::OpenSSL => "openssl",
            Self::GnuTLS => "gnutls",
            Self::MbedTLS => "mbedtls",
            Self::RusTLS => "rustls",
        }
        .to_string()
    }
}
