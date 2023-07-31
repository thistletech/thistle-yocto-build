use anyhow::*;
use std::{fs::File, io::Write, path::PathBuf};

use crate::{
    config::{Config, BUILD_ENV},
    credentials::Credentials,
    crypt3::crypt3_sha256_escaped,
    log,
};

// local.conf
pub fn local_conf_write(conf: &Config, debug: bool, c: Option<Credentials>) -> Result<()> {
    let p = BUILD_ENV.conf_dir.join("local.conf");
    let mut file = File::create(p)?;
    let mut w = Vec::new();
    writeln!(w, "MACHINE ??= \"{}\"", conf.machine)?;
    writeln!(w, "DISTRO ??= \"{}\"", conf.distro)?;
    writeln!(w, "SDKMACHINE = \"x86_64\"")?;
    writeln!(w)?;

    // Enable build history which outputs all sorts of interesting build info such as the dependency graph and a complete file manifest
    writeln!(w, "INHERIT += \"buildhistory\"")?;
    writeln!(w, "BUILDHISTORY_COMMIT = \"0\"")?;
    writeln!(w, "BUILDHISTORY_FEATURES = \"image\"")?;
    writeln!(w)?;

    if let Some(tf) = &conf.thistle_features {
        tf.write_local_conf(&mut w)?;
    }

    if debug {
        writeln!(w, "IMAGE_FEATURES:append = \" ssh-server-openssh\"")?;
        writeln!(w, "IMAGE_INSTALL:append = \" ssh-pregen-hostkeys\"")?;

        if let Some(creds) = &c {
            let username = &creds.username;
            let password = creds.password.clone();
            let salt = creds.salt.clone();
            let passwd = crypt3_sha256_escaped(password, salt).unwrap();

            writeln!(w, "INHERIT += \"extrausers\"")?;
            writeln!(w, "EXTRA_USERS_PARAMS = \"useradd -p '{passwd}' -G sudo {username};\"")?;
        }
    }

    if conf.ccache.unwrap_or(true) {
        log!("Enabling ccache");
        writeln!(w, "INHERIT += \"ccache\"")?;
    }

    writeln!(w, "IMAGE_INSTALL:append = \" sudo\"")?;

    // // enable CVE checking on non debug build
    // if !self.build_options.debug {
    //     writeln!(w, "INHERIT += \"cve-check\"")?;
    // }

    log!("Setting up disk monitoring system ");

    let monitoring = r#"# disk monitoring
BB_DISKMON_DIRS = "\
    STOPTASKS,${TMPDIR},1G,100K \
    STOPTASKS,${DL_DIR},1G,100K \
    STOPTASKS,${SSTATE_DIR},1G,100K \
    STOPTASKS,/tmp,100M,100K""#;

    writeln!(w, "{monitoring}")?;

    // auto detect CPUS
    let cpus = num_cpus::get();
    log!("Detected {} CPUs, will configure build accordingly ", cpus);

    writeln!(w, "BB_NUMBER_THREADS = \"{cpus}\"")?;
    writeln!(w, "PARALLEL_MAKE = \"-j {cpus}\"")?;

    // add all custom fields from local_conf
    if let Some(conf) = &conf.local_conf {
        for (k, v) in conf {
            writeln!(w, "# {k}")?;
            writeln!(w, "{v}")?;
        }
    }

    file.write_all(w.as_slice())?;
    file.flush()?;
    Ok(())
}

// bblayers.conf
pub fn bblayer_conf_write(conf: &Config, layers: Vec<PathBuf>) -> Result<()> {
    let p = BUILD_ENV.conf_dir.join("bblayers.conf");
    let mut file = File::create(p)?;
    let mut w = Vec::new();

    if let Some(conf) = &conf.bblayers_conf {
        for (k, v) in conf {
            writeln!(w, "# {k}")?;
            writeln!(w, "{v}")?;
        }
    }

    let layers: Vec<String> = layers.iter().map(|l| l.to_string_lossy().to_string()).collect();
    let layers_str = layers.join(" \\\n    ");

    write!(w, "BBLAYERS ?= \" \\\n    {layers_str}")?;
    writeln!(w, "\"")?;

    writeln!(w, "BBPATH ?= \"${{TOPDIR}}\"")?;
    writeln!(w, "BBFILES ??= \"\"")?;

    file.write_all(w.as_slice())?;
    file.flush()?;
    Ok(())
}

// site.conf
pub fn site_conf_write() -> Result<()> {
    let p = BUILD_ENV.conf_dir.join("site.conf");
    let mut file = File::create(p)?;
    let mut w = Vec::new();

    writeln!(w, "SCONF_VERSION = \"1\"")?;
    writeln!(w, "DEPLOY_DIR = \"{}\"", BUILD_ENV.deploy_dir.to_string_lossy())?;

    file.write_all(w.as_slice())?;
    file.flush()?;
    Ok(())
}
