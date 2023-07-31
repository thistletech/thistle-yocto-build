use std::io::Write;

pub mod curl;
pub mod libc;
pub mod tls;

use crate::config::features::libc::Libc;
use serde::{Deserialize, Serialize};
use tls::TLS;

use self::curl::Curl;

impl Features {
    pub fn write_local_conf(&self, w: &mut Vec<u8>) -> Result<(), std::io::Error> {
        if self.updater {
            writeln!(w, "IMAGE_INSTALL:append = \" embedded-client\"")?;
        }

        if self.read_only_rootfs {
            writeln!(w, "IMAGE_FEATURES:append = \" read-only-rootfs\"")?;
        }

        if let Some(curl) = &self.curl {
            writeln!(w, "{}", curl.to_image_install_value())?;
        }

        match self.libc {
            Some(Libc::Musl) => writeln!(w, "TCLIBC = \"musl\"")?,
            Some(Libc::Glibc) => writeln!(w, "TCLIBC = \"glibc\"")?,
            None => (),
        }
        Ok(())
    }
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Features {
    #[serde(rename = "meta-thistle")]
    pub meta_thistle: String,
    #[serde(default)]
    #[serde(rename = "read-only-rootfs")]
    pub read_only_rootfs: bool,
    #[serde(default)]
    pub updater: bool,
    pub libc: Option<Libc>,
    pub curl: Option<Curl>,
}
