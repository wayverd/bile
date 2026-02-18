use std::path::PathBuf;

use clap::Parser as _;
use figment::{
    Figment,
    providers::{Env, Format, Serialized, Toml},
};

#[derive(Debug, Clone, clap::Parser, serde::Serialize, serde::Deserialize)]
#[command(version, about)]
pub struct Config {
    /// The port the server will listen on
    #[arg(short, long, default_value_t = default_port())]
    pub port: u16,

    /// Directory to find git repos
    #[arg(short = 'r', long, default_value = default_repo_directory().into_os_string())]
    pub project_root: PathBuf,

    /// The text shown in a browsers title bar
    #[arg(short, long, default_value_t = default_site_name())]
    pub site_name: String,

    /// File to check for in the .git directory to decide wether to publicly show a repo
    #[arg(short, long, default_value_t = default_export_ok())]
    pub export_ok: String,

    /// Base URL to clone repositories from (without trailing slash)
    #[arg(short, long, default_value_t = String::new())]
    pub clone_base: String,

    /// Number of commits to be shown when paginating the log
    #[arg(short, long, default_value_t = default_log_per_page())]
    pub log_per_page: usize,
}

impl Config {
    pub fn load() -> crate::utils::error::Result<Self> {
        let config: Self = Figment::new()
            .merge(Serialized::defaults(Self::parse()))
            .merge(Toml::file("bile.toml"))
            .merge(Env::prefixed("BILE_"))
            .extract()?;

        Ok(config)
    }

    pub fn finalize(self) -> crate::utils::error::Result<Self> {
        Ok(Self {
            port: self.port,
            project_root: self.project_root.canonicalize()?,
            site_name: self.site_name,
            export_ok: self.export_ok,
            clone_base: self.clone_base,
            log_per_page: self.log_per_page,
        })
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: default_port(),
            project_root: default_repo_directory(),
            site_name: default_site_name(),
            export_ok: default_export_ok(),
            clone_base: String::new(),
            log_per_page: default_log_per_page(),
        }
    }
}

const fn default_port() -> u16 {
    80
}

fn default_repo_directory() -> PathBuf {
    PathBuf::from("./repos")
}

fn default_site_name() -> String {
    "bile".to_string()
}

fn default_export_ok() -> String {
    "git-daemon-export-ok".to_string()
}

const fn default_log_per_page() -> usize {
    100
}
