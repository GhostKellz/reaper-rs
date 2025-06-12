pub mod aur;
pub mod backend;
pub mod config;
pub mod core;
pub mod flatpak;
pub mod gpg;
pub mod hooks;
pub mod pacman;
pub mod tui;
pub mod utils;

pub use crate::aur::get_deps;
pub use crate::aur::SearchResult;
pub use crate::core::{Source, install_with_priority, unified_search};
// pub use crate::backend; // Removed to avoid E0255 duplicate name conflict
