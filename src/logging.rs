//! Handling of debug logging.
//!
//! # Debug logging
//!
//! Since all forms of output are used by the application, the logging can only be performed
//! into a file. By default, the logging is turned off and it can be turned on by setting the
//! environment varible MLESS_LOG to the path of the file in which to log:
//!
//! ```
//! $ MLESS_LOG=/tmp/log.log mless file_to_select_from.txt
//! ```
//!
//! By default, debug level and higher are logged. To customize the log levels, set RUST_LOG
//! according to [env_logger's documentation](https://docs.rs/env_logger/0.11.3/env_logger/index.html).
use std::fs::File;

use env_logger::Env;
use log::info;
use snafu::ResultExt;

use crate::error::LoggingStartSnafu;
use crate::error::RunError;

/// Name for the environment variable containing the path of the log file.
const LOG_PATH_ENV: &str = "MLESS_LOG";
/// Default logging level if RUST_LOG is not provided.
const LOG_DEFAULT_LEVEL: &str = "debug";

/// Initialize the logging according to environment variables.
/// Returns an error if it cannot open the log file for writing.
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
