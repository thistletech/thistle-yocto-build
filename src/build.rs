#![allow(clippy::field_reassign_with_default)]
use crate::config::{Config, BUILD_ENV};
use crate::credentials::Credentials;
use crate::repo::Repo;
use crate::yocto_configs;
use crate::{log, log_warn};
use anyhow::{Ok, *};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const BITBAKE_REFSPEC: &str = "2022-04.6-kirkstone";
const BITBAKE_URL: &str = "git://git.openembedded.org/bitbake";

const META_THISTLE_URL: &str = "https://github.com/thistletech/meta-thistle.git";

pub fn find_bitbake_in_top_layers(layer_abs_paths: Vec<PathBuf>) -> Option<PathBuf> {
    for path in layer_abs_paths {
        let path = path.join("bitbake");
        if path.is_dir() {
            return Some(path);
        }
    }
    None
}

fn set_thistle(hash: &str) -> Repo {
    let layers = vec![
        "meta-thistle-base".to_string(),
        "meta-thistle-base-bsp".to_string(),
        "meta-thistle-update-client".to_string(),
        "meta-trust-m".to_string(),
    ];

    Repo {
        name: "meta-thistle".to_string(),
        layers,
        location: META_THISTLE_URL.to_string(),
        refspec: Some(hash.to_string()),
    }
}

fn find_oe_init() -> Result<PathBuf> {
    for entry in fs::read_dir(&BUILD_ENV.layer_dir)? {
        let path = entry?.path();
        if path.is_dir() {
            let oe_init = path.join("oe-init-build-env");
            if oe_init.exists() {
                return Ok(oe_init);
            }
        }
    }
    bail!("Unable to find oe-init-build-env in any of the layers")
}

pub fn build(
    config_file: &Path,
    debug: bool,
    dryrun: bool,
    bitbake_extra_args: &Option<String>,
) -> Result<()> {
    let config = Config::parse_from_path(config_file)?;
    let extra_bbargs = bitbake_extra_args.clone().unwrap_or_default();
    let mut repos = config.repos.clone();

    if debug {
        log_warn!("Building in insecure debug mode");
    }

    let dryrun_flag = if dryrun {
        log_warn!("running in dry run mode");
        "--dry-run"
    } else {
        ""
    };

    let creds = match debug {
        true => Some(Credentials::new()?),
        false => None,
    };

    if let Some(f) = &config.thistle_features {
        let thistle_repo = set_thistle(&f.meta_thistle);
        repos.push(thistle_repo);
    }

    // Fetch repos and parse layer configuration
    let (top_layers, layers) = crate::repo::fetch_repos(&repos).context("Unable to fetch repos")?;

    // See if bitbake is already here, or otherwise default to fetching it
    if let Some(_bitbake_path) = find_bitbake_in_top_layers(top_layers) {
        log!("Found bitbake on top layer");
    } else {
        log!("Fetching bitbake from upstream git repo at commit {}", &BITBAKE_REFSPEC);
        let r = Repo {
            name: "bitbake".to_string(),
            layers: Vec::new(),
            location: BITBAKE_URL.to_string(),
            refspec: Some(BITBAKE_REFSPEC.to_string()),
        };
        r.fetch().context("unable to fetch bitbake")?;
    }

    let oe = find_oe_init()?;
    log!("Found oe-init-build-env at {:?}", oe);

    // Create configuration files
    log!("Generating bitbake config files");
    yocto_configs::local_conf_write(&config, debug, creds).context("unable to write conf file")?;
    yocto_configs::bblayer_conf_write(&config, layers).context("unable to write conf file")?;
    yocto_configs::site_conf_write().context("unable to write conf file")?;

    let build_script_path = "./.thistlebuild.sh";
    let source_oe = format!("source {oe:?} > /dev/null");
    let bb_cmd = format!("bitbake {} {} -k {}", extra_bbargs, dryrun_flag, config.target);
    let exec_script = format!("{} && {}", source_oe, bb_cmd);

    log!("Executing {bb_cmd:?}");
    log!("\nBuild starting - {:?}\n", chrono::offset::Local::now());
    let ts = chrono::offset::Local::now().timestamp();

    fs::write(build_script_path, exec_script).context("can't write build file")?;
    let build_err = "can't run build script";

    let cmd = Command::new("bash")
        .arg(build_script_path)
        .spawn()
        .context(build_err)?
        .wait()
        .context(build_err)?;

    let ts_end = chrono::offset::Local::now().timestamp();

    if dryrun {
        log!("\nDry run done in {}s", ts_end - ts);
        return Ok(());
    }

    log!("\nBuild done in {}s", ts_end - ts);

    if !cmd.success() {
        crate::common::notify_user("Build failed", "error");
        return Ok(());
    }

    crate::common::notify_user("Build successful", "information");
    log!("Build artifact folder: {:?}", BUILD_ENV.deploy_dir);

    if let Err(e) = check_for_insecure_features(&config) {
        log_warn!("Failed scanning build for insecure features");
        log!("{:?}", e);
    }

    Ok(())
}

fn check_for_insecure_features(config: &Config) -> Result<()> {
    log!("Analyzing build for security issues...");

    let test_data_path = format!(
        "build/deploy/images/{}/{}-{}.testdata.json",
        &config.machine, &config.target, &config.machine
    );

    let td = std::fs::read_to_string(test_data_path).context("failed to locate build metatada")?;
    let test_data_json: serde_json::Value = serde_json::from_str(&td)?;
    let test_data_map = test_data_json.as_object().ok_or(anyhow!("unable to parse local data"))?;

    let image_features =
        test_data_map.get("IMAGE_FEATURES").ok_or(anyhow!("unable to find image features"))?;

    let image_features: Vec<&str> = match image_features {
        serde_json::Value::String(s) => s.split_ascii_whitespace().collect(),
        _ => panic!("Unsupported json type"),
    };

    let insecure_image_features = vec![
        "allow-empty-password",
        "debug-tweaks",
        "empty-root-password",
        "post-install-logging",
    ];

    let mut insecure_image_feature_found = false;
    for insecure_image_feature in insecure_image_features {
        if image_features.contains(&insecure_image_feature) {
            log_warn!("Found insecure image feature: {}", &insecure_image_feature);
            insecure_image_feature_found = true;
        }
    }

    if !insecure_image_feature_found {
        log!("No issues detected");
    }
    Ok(())
}
