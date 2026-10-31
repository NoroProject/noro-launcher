//! `--key value` flags. Six of them don't justify a CLI framework.

use anyhow::{bail, Result};
use std::path::PathBuf;

#[derive(Default)]
pub struct Args {
    pub name: Option<String>,
    pub targets: Vec<String>,
    pub work: Option<PathBuf>,
    pub repo: Option<PathBuf>,
    pub deploy: Option<String>,
    pub no_wait: bool,
}

impl Args {
    pub fn parse(mut it: impl Iterator<Item = String>) -> Result<Self> {
        let mut a = Args::default();
        while let Some(flag) = it.next() {
            if flag == "--no-wait" {
                a.no_wait = true;
                continue;
            }
            let Some(value) = it.next() else {
                bail!("{flag} needs a value");
            };
            match flag.as_str() {
                "--name" => a.name = Some(value),
                "--targets" => {
                    a.targets = value
                        .split(',')
                        .map(str::trim)
                        .filter(|t| !t.is_empty())
                        .map(String::from)
                        .collect()
                }
                "--work" => a.work = Some(value.into()),
                "--repo" => a.repo = Some(value.into()),
                "--deploy" if value == "core" || value == "all" => a.deploy = Some(value),
                "--deploy" => bail!("--deploy takes core or all"),
                _ => bail!("unknown flag {flag}"),
            }
        }
        Ok(a)
    }
}
