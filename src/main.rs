mod models;
mod parsers;
mod theme;
mod ui;
mod utils;

use gtk4::prelude::*;
use gtk4::Application;
use models::RunMode;
use std::path::PathBuf;

/// Парсит аргументы командной строки
fn parse_args() -> RunMode {
    let args: Vec<String> = std::env::args().collect();
    let mut mode = RunMode::All;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--hyprland" => {
                if i + 1 < args.len() && !args[i + 1].starts_with("--") {
                    mode = RunMode::SingleHyprland(PathBuf::from(&args[i + 1]));
                    i += 1;
                } else {
                    // Default: auto-detect in ~/.config/hypr/
                    let hypr_dir = dirs::config_dir()
                        .map(|p| p.join("hypr"))
                        .unwrap_or_else(|| PathBuf::from("."));
                    let lua = hypr_dir.join("hyprland.lua");
                    let conf = hypr_dir.join("hyprland.conf");
                    if lua.exists() {
                        mode = RunMode::SingleHyprland(lua);
                    } else {
                        mode = RunMode::SingleHyprland(conf);
                    }
                }
            }
            "--sxhkd" => {
                if i + 1 < args.len() && !args[i + 1].starts_with("--") {
                    mode = RunMode::SingleSxhkd(PathBuf::from(&args[i + 1]));
                    i += 1;
                } else {
                    let default = dirs::config_dir()
                        .map(|p| p.join("sxhkd/sxhkdrc"))
                        .unwrap_or_else(|| PathBuf::from("sxhkdrc"));
                    mode = RunMode::SingleSxhkd(default);
                }
            }
            _ => {}
        }
        i += 1;
    }

    mode
}

fn main() {
    let mode = parse_args();

    let app = Application::builder()
        .application_id("com.meowrch.HotkeyHub")
        .build();
    
    app.connect_activate(move |app| {
        ui::build_ui(app, &mode);
    });

    app.run_with_args::<&str>(&[]);
}
