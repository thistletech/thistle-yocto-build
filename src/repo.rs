use crate::config::BUILD_ENV;
use crate::{log, log_warn};
use anyhow::*;
use serde::Deserialize;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Deserialize, Default, Clone)]
pub struct Repo {
    #[serde(skip)]
    pub name: String,
    #[serde(default)]
    pub layers: Vec<String>,

    #[serde(alias = "url", alias = "path")]
    pub location: String,

    // git only
    pub refspec: Option<String>,
}

fn rungit(path: &PathBuf, args: &[&str]) -> Result<String> {
    let c = Command::new("git")
        .current_dir(path)
        .args(args)
        .output()
        .context("Failed to run git command")?;
    let stdout = String::from_utf8_lossy(&c.stdout).trim().to_string();
    if !c.status.success() {
        eprintln!("stdout: {}", stdout);
        eprintln!("stderr: {}", String::from_utf8_lossy(&c.stderr));
        bail!("Failed to run git command");
    }
    Ok(stdout)
}

impl Repo {
    pub fn is_local_repo(&self) -> bool {
        PathBuf::from(&self.location).exists()
    }

    pub fn path(&self) -> PathBuf {
        let p = if self.is_local_repo() {
            // local repo - use path directly
            PathBuf::from(&self.location)
        } else {
            // we buffer the repote repo in the layers folder
            BUILD_ENV.layer_dir.join(&self.name)
        };

        p.canonicalize().unwrap_or(p)
    }

    pub fn fetch(&self) -> Result<()> {
        if self.is_local_repo() {
            log!("Scanning local repository {}", self.name);
            return Ok(()); // local repo
        }

        if self.refspec.is_none() {
            bail!("Missing refspec for repository: {}", self.name)
        }

        let refspec = self.refspec.clone().unwrap_or_default();
        let url = self.location.clone();
        let path = self.path();

        if !path.exists() {
            log!("Cloning remote repo {}", self.name);
            let path_str = path.to_string_lossy().to_string();
            std::fs::create_dir_all(&path).context("can't create repo folder")?;
            rungit(&path, &["clone", &url, &path_str])?;
        } else {
            let repo_not_tainted = rungit(&path, &["status", "--porcelain"])?.is_empty();
            if !repo_not_tainted {
                log_warn!("Repository {} is locally changed, not checking out ref {}", self.name, &refspec);
                return Ok(());
            }

            log!("Updating remote repo {}", self.name);
            let main = rungit(&path, &["symbolic-ref", "--short", "refs/remotes/origin/HEAD"])?;
            let main = main.replace("origin/", "");
            rungit(&path, &["checkout", &main])?;
            rungit(&path, &["pull"])?;
        }

        rungit(&path, &["checkout", &refspec])?;
        Ok(())
    }
}

pub fn fetch_repos(repos: &Vec<Repo>) -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    let mut top_layers = Vec::new();
    let mut layers = Vec::new();
    for r in repos {
        r.fetch()?;
        top_layers.push(r.path());

        if r.layers.is_empty() {
            layers.push(r.path());
        } else {
            for layer in &r.layers {
                let layer_path = r.path().join(layer);
                layers.push(layer_path);
            }
        }
    }

    Ok((top_layers, layers))
}
