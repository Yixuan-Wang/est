use std::{env, path::PathBuf};

fn locate_config_file() -> Option<PathBuf> {
    let path_config_home = env::var("XDG_CONFIG_HOME")
        .map(|home| {
            let mut path = PathBuf::from(home);
            path.push("est");
            path.push("config.toml");
            path
        })
        .ok();

    let path_current = env::current_dir()
        .map(|dir| {
            let mut path = dir;
            path.push("config.toml");
            path
        })
        .ok();

    [path_current, path_config_home]
        .into_iter()
        .flatten()
        .find(|f| f.exists())
}

pub fn build() -> est_core::Instance {
    let path = locate_config_file().expect("Cannot find config.toml");
    let config = std::fs::read_to_string(&path).expect("Cannot read config.toml");

    let compose = toml::from_str::<est_core::compose::Compose>(&config).unwrap_or_else(|err| {
        panic!("Failed to parse config file: {}", err);
    });

    est_core::Instance::from(compose)
}
