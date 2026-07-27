use crate::models::Config;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

/// Загружает тему из конфигурационного файла
pub fn load_config() -> Config {
    let mut config = Config::default();
    let config_path = dirs::config_dir()
        .map(|p| p.join("HotkeyHub/app.conf"))
        .unwrap_or_else(|| PathBuf::from("app.conf"));

    if let Ok(file) = File::open(&config_path) {
        let reader = BufReader::new(file);
        let re = Regex::new(r"^\s*([a-z_]+)\s*=\s*(.+)$").unwrap();

        for line in reader.lines().filter_map(|l| l.ok()) {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(caps) = re.captures(line) {
                let key = &caps[1];
                let val = caps[2].trim().to_string();
                match key {
                    "switch_description" => config.switch_description = val.parse().unwrap(),
                    _ => {}
                }
            }
        }
    }
    config
}