//! Configuration loading and showing.

use std::{fs::File, path::PathBuf};

use snafu::ResultExt;

use crate::{
    configuration::Config,
    error::{ConfigOpenSnafu, ConfigParseSnafu, RunError},
};

/// Load the [Config] from the given path. If path is [None], the default
/// value for [Config] is returned.
pub fn load_config(path: Option<PathBuf>) -> Result<Config, RunError> {
    if let Some(path) = path {
        let file = File::open(path.clone()) //
            .context(ConfigOpenSnafu { path: path.clone() })?;
        let config = Config::try_from(file) //
            .context(ConfigParseSnafu { path })?;

        return Ok(config);
    }

    Ok(Config {
        ..Default::default()
    })
}
