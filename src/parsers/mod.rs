pub mod hyprland;
pub mod hyprland_lua;
pub mod sxhkd;

pub use hyprland::parse_hyprland_recursive;
pub use hyprland_lua::parse_hyprland_lua_recursive;
pub use sxhkd::parse_sxhkd;
