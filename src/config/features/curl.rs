use crate::config::features::TLS;
use serde::de::Error;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::str::FromStr;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Curl {
    pub bin: bool,
    pub lib: bool,
    pub tls: TLS,
}

impl TryFrom<&Value> for Curl {
    type Error = serde_yaml::Error;
    fn try_from(v: &Value) -> Result<Self, Self::Error> {
        let map = v
            .as_sequence()
            .unwrap()
            .first()
            .unwrap()
            .as_mapping()
            .unwrap()
            .get(&Value::String("curl".to_string()))
            .unwrap()
            .as_mapping()
            .unwrap();
        let mut lib = false;
        let mut bin = false;
        let mut tls = None;
        for (k, v) in map.iter() {
            let k = k.as_str().ok_or_else(|| {
                serde_yaml::Error::custom("Unexpected field type for 'curl'. Must be a string.")
            })?;
            match k {
                "lib" => {
                    lib = v.as_bool().ok_or_else(|| {
                        serde_yaml::Error::custom("Unexpected field type for 'curl' lib. Must be a bool.")
                    })?;
                }
                "bin" => {
                    bin = v.as_bool().ok_or_else(|| {
                        serde_yaml::Error::custom("Unexpected field type for 'curl' bin. Must be a bool.")
                    })?;
                }
                "tls" => {
                    let tls_str = v.as_str().ok_or_else(|| {
                        serde_yaml::Error::custom("Unexpected field type for 'curl' tls. Must be a string.")
                    })?;
                    tls = Some(TLS::from_str(tls_str)?);
                }
                _ => {
                    bail_de!("Unsupported key '{}' found for 'curl'", k);
                }
            }
        }

        if !lib && !bin {
            bail_de!("Attempting to include 'curl' without bin or lib flags set");
        }

        let tls = match tls {
            Some(TLS::BearSSL) | Some(TLS::WolfSSL) => {
                bail_de!("The selected TLS stack is not supported by 'curl'");
            }
            Some(t) => t,
            None => bail_de!("A TLS stack must be selected for 'curl'"),
        };

        let curl_imp = Curl { bin, lib, tls };
        Ok(curl_imp)
    }
}

impl Curl {
    pub fn to_image_install_value(&self) -> String {
        let mut ret = "IMAGE_INSTALL:append = \" ".to_string();
        if self.bin {
            ret.push_str("curl");
        }
        if self.lib {
            ret.push_str(" libcurl");
        }

        ret.push_str("\"\n");
        ret.push_str(&format!("PACKAGECONFIG:append:pn-curl = \" {}\"", self.tls.to_image_install_value()));
        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_config() {
        let example = r#"
        - curl:
            lib: true
            bin: true
            tls: "rustls"
        "#;
        let example_yaml: serde_yaml::Value = serde_yaml::from_str(example).unwrap();
        let curl = Curl::try_from(&example_yaml).unwrap();
        let expected_output = "IMAGE_INSTALL:append = \" curl libcurl\"\n\
        PACKAGECONFIG:append:pn-curl = \" rustls\"";
        let value = curl.to_image_install_value();
        assert_eq!(&value, expected_output);
    }
}
