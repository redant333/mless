//! Configuration loading and showing.

use std::{
    env::{self, VarError},
    fs::File,
    path::{Path, PathBuf},
};

use snafu::ResultExt;

use crate::{
    configuration::Config,
    error::{ConfigOpenSnafu, ConfigParseSnafu, RunError},
};

/// Get the absolute path of the file in `$environment_variable_dir/$path` if
/// the environment variable can be read and if the file exists.
///
/// Note that this function does not check whether the file is readable,
/// only wheter it exists.
fn get_config_from(
    environment_variable_dir: &str,
    path: &str,
    file_exists: &dyn Fn(&str) -> bool,
    get_env_var: &dyn Fn(&str) -> Result<String, VarError>,
) -> Option<PathBuf> {
    let directory = get_env_var(environment_variable_dir).ok()?;
    let config_path = format!("{directory}/{path}");

    if file_exists(&config_path) {
        Some(PathBuf::from(config_path))
    } else {
        None
    }
}

#[allow(
    clippy::needless_match,
    reason = "False positive https://github.com/rust-lang/rust-clippy/issues/13574"
)]
#[allow(
    clippy::manual_map,
    reason = "Using this to be extra explicit about the priority of configs"
)]
/// Implementation of [get_config_file_location] with additional arguments
/// to make testing easier. See [get_config_file_location] for details.
///
/// Arguments:
///  - `file_exists`: function to use to check if the file with the given path exists.
///    Should return true when it does and false otherwise,
///  - `get_env_var`: function to use to get the value of the the given environment variable.
///    Should return the value of the variable with the given name if the variable can be read
///    or an error otherwise.
fn get_config_file_location_impl(
    file_exists: &dyn Fn(&str) -> bool,
    get_env_var: &dyn Fn(&str) -> Result<String, VarError>,
) -> Option<PathBuf> {
    let get_config_from = |env, path| get_config_from(env, path, file_exists, get_env_var);

    if let Some(path) = get_config_from("XDG_CONFIG_HOME", "mless/mless.yaml") {
        Some(path)
    } else if let Some(path) = get_config_from("HOME", ".config/mless/mless.yaml") {
        Some(path)
    } else if let Some(path) = get_config_from("HOME", ".mless.yaml") {
        Some(path)
    } else {
        None
    }
}

/// Get the config file location.
///
/// The following paths are checked in this order:
/// - `$XDG_CONFIG_HOME/mless/mless.yaml`
/// - `$HOME/.config/mless/mless.yaml`
/// - `$HOME/.mless.yaml`
///
/// The first two are defined in [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/latest/#variables)
/// and the third one is a common configuration pattern.
///
/// Returns the absolute path of first configuration file that exists or None
/// if none of them do.
pub fn get_config_file_location() -> Option<PathBuf> {
    let file_exists = |path: &str| Path::new(&path).exists();
    let get_env_var = |var_name: &str| env::var(var_name);

    get_config_file_location_impl(&file_exists, &get_env_var)
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(
        Ok("/home/user/.xdg_config".to_string()),
        Ok("/home/user".to_string()),
        &["/home/user/.xdg_config/mless/mless.yaml"],
        PathBuf::from("/home/user/.xdg_config/mless/mless.yaml"); "when_only_xdg_config_file_exists")]
    #[test_case(
        Ok("/home/user/.xdg_config".to_string()),
        Ok("/home/user".to_string()),
        &["/home/user/.config/mless/mless.yaml"],
        PathBuf::from("/home/user/.config/mless/mless.yaml"); "when_only_home_config_file_exists")]
    #[test_case(
        Ok("/home/user/.xdg_config".to_string()),
        Ok("/home/user".to_string()),
        &["/home/user/.mless.yaml"],
        PathBuf::from("/home/user/.mless.yaml"); "when_only_home_file_exists")]
    #[test_case(
        Err(VarError::NotPresent),
        Ok("/home/user".to_string()),
        &["/home/user/.config/mless/mless.yaml"],
        PathBuf::from("/home/user/.config/mless/mless.yaml"); "when_xdg_config_is_not_defined_and_home_config_exists")]
    #[test_case(
        Err(VarError::NotPresent),
        Ok("/home/user".to_string()),
        &["/home/user/.mless.yaml"],
        PathBuf::from("/home/user/.mless.yaml"); "when_xdg_config_is_not_defined_and_home_does_not_exist")]
    #[test_case(
        Ok("/home/user/.xdg_config".to_string()),
        Ok("/home/user".to_string()),
        &["/home/user/.xdg_config/mless/mless.yaml", "/home/user/.config/mless/mless.yaml"],
        PathBuf::from("/home/user/.xdg_config/mless/mless.yaml"); "when_xdg_config_and_home_config_exist")]
    #[test_case(
        Ok("/home/user/.xdg_config".to_string()),
        Ok("/home/user".to_string()),
        &["/home/user/.xdg_config/mless/mless.yaml", "/home/user/.mless.yaml"],
        PathBuf::from("/home/user/.xdg_config/mless/mless.yaml"); "when_xdg_config_and_home_file_exist")]
    #[test_case(
        Ok("/home/user/.xdg_config".to_string()),
        Ok("/home/user".to_string()),
        &["/home/user/.config/mless/mless.yaml", "/home/user/.mless.yaml"],
        PathBuf::from("/home/user/.config/mless/mless.yaml"); "when_home_config_and_home_file_exist")]
    #[test_case(
        Ok("/home/user/.xdg_config".to_string()),
        Ok("/home/user".to_string()),
        &["/home/user/.config/mless/mless.yaml", "/home/user/.xdg_config/mless/mless.yaml", "/home/user/.mless.yaml"],
        PathBuf::from("/home/user/.xdg_config/mless/mless.yaml"); "when_all_three_files_exist")]
    fn get_config_file_location_returns_expected_value(
        xdg_config_home: Result<String, VarError>,
        home: Result<String, VarError>,
        existing_files: &[&str],
        expected: PathBuf,
    ) {
        let fake_file_exists = |path: &str| existing_files.contains(&path);
        let fake_get_env_var = |var_name: &str| match var_name {
            "XDG_CONFIG_HOME" => xdg_config_home.clone(),
            "HOME" => home.clone(),
            _ => Err(VarError::NotPresent),
        };

        let config_path =
            get_config_file_location_impl(&fake_file_exists, &fake_get_env_var).unwrap();

        assert_eq!(config_path, expected);
    }

    #[test]
    fn get_config_file_location_returns_none_if_no_files_exist() {
        let fake_file_exists = |_path: &str| false;
        let fake_get_env_var = |_var_name: &str| Ok("/path/not/important".to_string());

        let config_path = get_config_file_location_impl(&fake_file_exists, &fake_get_env_var);

        assert!(config_path.is_none());
    }
}
