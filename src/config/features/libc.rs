use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::str::FromStr;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Libc {
    Musl,
    Glibc,
}

impl Default for Libc {
    fn default() -> Self {
        Self::Glibc
    }
}

impl FromStr for Libc {
    type Err = serde_yaml::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let result = match value {
            "musl" => Self::Musl,
            "glibc" => Self::Glibc,
            _ => bail_de!("Invalid libc type: {}", value),
        };
        Ok(result)
    }
}

impl Libc {
    pub fn parse(v: &Value) -> Result<Self, serde_yaml::Error> {
        match v {
            Value::String(v) => Self::from_str(v),
            _ => bail_de!("Unexpected field type for 'libc'"),
        }
    }
}
