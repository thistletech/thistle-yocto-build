#[macro_use]
mod common;
pub mod error;
pub mod features;

use anyhow::*;
use serde::Deserialize;
use std::fmt::Debug;
use std::path::Path;
use std::{collections::BTreeMap, path::PathBuf};

use crate::repo::Repo;

use self::features::Features;

pub struct BuildEnv {
    pub work_dir: PathBuf,
    pub layer_dir: PathBuf,
    pub deploy_dir: PathBuf,
    pub conf_dir: PathBuf,
}

lazy_static! {
    pub static ref BUILD_ENV: BuildEnv = {
        let work_dir = PathBuf::from("./build");
        std::fs::create_dir_all(&work_dir).expect("can't handle directory ./build, check permissions");
        let work_dir = work_dir.canonicalize().expect("can't handle directory ./build, check permissions");

        let prepare_path = |t: &str| -> PathBuf {
            let err = format!("can't handle directory ./build/{t}, check permissions");
            let p = work_dir.join(t);
            std::fs::create_dir_all(&p).expect(&err);
            p.canonicalize().expect(&err)
        };

        let layer_dir = prepare_path("layers");
        let deploy_dir = prepare_path("deploy");
        let conf_dir = prepare_path("conf");

        BuildEnv {
            work_dir,
            layer_dir,
            deploy_dir,
            conf_dir,
        }
    };
}

impl Config {
    pub fn parse_from_path(config_path: &Path) -> Result<Self> {
        let cantfind = "Unable to find config file path";
        let config_file_path = std::fs::canonicalize(config_path).context(cantfind)?;
        let config_file = std::fs::File::open(config_file_path).context(cantfind)?;

        let cantparse = "Unable to parse config file";
        let record: Self = serde_yaml::from_reader(config_file).context(cantparse)?;
        Ok(record)
    }
}

fn parse_repos<'de, D>(deserializer: D) -> std::result::Result<Vec<Repo>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let mut repos: Vec<Repo> = Vec::new();
    let map = BTreeMap::<String, Repo>::deserialize(deserializer)?;

    for (name, mut repo) in map {
        repo.name = name;
        repos.push(repo);
    }

    std::result::Result::Ok(repos)
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub target: String,
    pub machine: String,
    pub distro: String,

    #[serde(rename = "thistle-features")]
    pub thistle_features: Option<Features>,
    pub ccache: Option<bool>,

    // deserialize this with a custom deserializer named bobdylan
    #[serde(deserialize_with = "parse_repos")]
    pub repos: Vec<Repo>,

    #[serde(rename = "local_conf_header")]
    pub local_conf: Option<BTreeMap<String, String>>,
    #[serde(rename = "bblayers_conf_header")]
    pub bblayers_conf: Option<BTreeMap<String, String>>,
}
