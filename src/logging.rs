use std::fs::File;

use env_logger::Env;
use log::info;
use snafu::ResultExt;

use crate::{LoggingStartSnafu, RunError};

const LOG_PATH_ENV: &str = "MLESS_LOG";
const LOG_DEFAULT_LEVEL: &str = "debug";

pub fn initialize_logging() -> Result<(), RunError> {
    let Ok(log_path) = std::env::var(LOG_PATH_ENV) else {
        return Ok(());
    };

    let log_file = File::create(log_path.clone()) //
        .context(LoggingStartSnafu {
            path: log_path.clone(),
        })?;

    let log_file = Box::new(log_file);

    env_logger::Builder::from_env(Env::default().default_filter_or(LOG_DEFAULT_LEVEL))
        .target(env_logger::Target::Pipe(log_file))
        .init();

    info!("Logging started");
    Ok(())
}
