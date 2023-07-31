use clap::{Parser, Subcommand};
use std::io::Write;
use std::path::PathBuf;

use anyhow::*;
use thistle_yocto_build::log;

#[derive(Parser, Debug)]
#[clap(
    name = "thistle yocto build",
    author,
    version,
    disable_help_subcommand = true,
    about = "Helper tool for yocto build system.\n
Examples:
    Initialize default config:   ./thistle-yocto-build gen-config qemu
    Start build:                 ./thistle-yocto-build build conf.yml
",
    verbatim_doc_comment,
    // term_width=110,
)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[clap(about = "Perform a yocto build")]
    Build(BuildArgs),
    #[clap(
        about = "Clean the build directory. Preserves sscache, and optionally the layers and downloads folders."
    )]
    Clean(CleanArgs),
    #[clap(subcommand, about = "Generate default configuration files")]
    GenConfig(ConfArg),
}

#[derive(Debug, Parser, Default)]
pub struct CleanArgs {
    /// Do not prompt for confirmation
    #[clap(short, long)]
    pub yes: bool,

    /// Preserve layers folder
    #[clap(short, long)]
    pub layers_keep: bool,

    /// Preserve downloads folder
    #[clap(short, long)]
    pub dl_keep: bool,
}

#[derive(Debug, Subcommand, PartialEq)]
pub enum ConfArg {
    /// qemu default config
    Qemu,
    /// raspberrypi 4 default config
    Rpi4,
    /// beagleboneblack default config
    Bbb,
}

#[derive(Debug, Parser, Default)]
pub struct BuildArgs {
    /// Extra arguments to pass to bitbake
    #[clap(short, long)]
    pub bitbake_extra_args: Option<String>,

    /// Enable debug features (insecure)
    #[clap(short, long)]
    pub debug: bool,

    /// Download assets but stop before performing the yocto build
    #[clap(long)]
    pub dryrun: bool,

    /// Configuration file to use
    pub config_file: PathBuf,
}

pub fn run_build(args: BuildArgs) -> Result<()> {
    log!("~~ Thistle Yocto Build Starting ~~");
    use thistle_yocto_build::build::build;
    build(&args.config_file, args.debug, args.dryrun, &args.bitbake_extra_args)?;
    Ok(())
}

fn clean(c: CleanArgs) -> Result<()> {
    if !c.yes {
        let mut empty = "".to_string();
        log!("WARNING: this will clean the build directory. Press any key to continue.");
        log!("NOTE: sstate-cache folder will be preserved");
        if c.layers_keep {
            log!("NOTE: layers folder will be preserved");
        }
        if c.dl_keep {
            log!("NOTE: downloads folder will be preserved");
        }
        std::io::stdin().read_line(&mut empty)?;
    }

    std::fs::rename("build", ".thistletmpbuild").context("build directory does not exist")?;
    std::fs::create_dir("build")?;

    std::fs::rename(".thistletmpbuild/sstate-cache", "build/sstate-cache")?;
    if c.layers_keep {
        std::fs::rename(".thistletmpbuild/layers", "build/layers")?;
    }
    if c.dl_keep {
        std::fs::rename(".thistletmpbuild/downloads", "build/downloads")?;
    }

    std::fs::remove_dir_all(".thistletmpbuild")?;

    log!("Cleanup done!");
    Ok(())
}

fn gen_config(c: ConfArg) -> Result<()> {
    let config_str = match c {
        ConfArg::Qemu => include_str!("../samples/qemuarm64.yml"),
        ConfArg::Rpi4 => include_str!("../samples/raspberrypi4.yml"),
        ConfArg::Bbb => include_str!("../samples/beagleboneblack.yml"),
    };

    let mut config_file = std::fs::File::create("conf.yml").context("Unable to create config file")?;
    config_file.write_all(config_str.as_bytes()).context("Unable to write config file")?;

    log!("Generated default thistle-yocto-build config file at conf.yml");
    log!("you can directly build this configuration by running './thistle-yocto-build build conf.yml'");

    if c == ConfArg::Qemu {
        log!("This qemu image can be emulated directly on your system using the following command (requres qemu-system-aarch64)");
        log!(
            r#"
qemu-system-aarch64 -machine virt -nographic -cpu cortex-a57 \
    -bios build/deploy/images/qemuarm64-thistle/u-boot.bin \
    -drive if=none,file=build/deploy/images/qemuarm64-thistle/base-qemuarm64-thistle.wic,id=disk0,format=raw \
    -device ich9-ahci,id=ahci -device ide-hd,drive=disk0,bus=ahci.0 -m 256M
"#
        );
    }

    Ok(())
}

pub fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Build(b) => run_build(b)?,
        Commands::Clean(c) => clean(c)?,
        Commands::GenConfig(c) => gen_config(c)?,
    }

    Ok(())
}
