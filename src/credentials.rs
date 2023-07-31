use std::io::Write;

use anyhow::*;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use rpassword::read_password;

pub struct Credentials {
    pub username: String,
    pub password: String,
    pub salt: String,
}

fn get_thistle_yocto_build_env_var(name: &str, private: bool) -> Result<String> {
    let name_uc_underscore = name.to_uppercase().replace(' ', "_");
    let key = format!("THISTLE_YOCTO_BUILD_{name_uc_underscore}");

    let value = std::env::var(&key).or_else(|_| {
        print!("The {key} environment variable was not set. Please enter a value for the {name}: ");
        std::io::stdout().flush()?;

        let r = if private {
            read_password()?
        } else {
            let mut value = String::new();
            std::io::stdin().read_line(&mut value)?;
            value.trim().to_string()
        };

        Ok(r)
    })?;

    Ok(value)
}

impl Credentials {
    pub fn new() -> Result<Self> {
        let username = get_thistle_yocto_build_env_var("username", false)?;
        let password = get_thistle_yocto_build_env_var("password", true)?;

        // use env salt if we have some
        let salt = std::env::var("THISTLE_YOCTO_BUILD_SALT").unwrap_or_else(|_| {
            let mut rng = thread_rng();
            let salt: String = (0..16).map(|_| rng.sample(Alphanumeric) as char).collect();
            salt
        });

        Ok(Self {
            username,
            password,
            salt,
        })
    }
}
