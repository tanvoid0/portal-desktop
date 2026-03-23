/**
 * Package Managers Module
 * 
 * All package manager implementations
 */

pub mod winget_manager;
pub mod scoop_manager;
pub mod chocolatey_manager;
pub mod cargo_manager;
pub mod homebrew_manager;
pub mod npm_manager;
pub mod pip_manager;

pub use winget_manager::WingetManager;
pub use scoop_manager::ScoopManager;
pub use chocolatey_manager::ChocolateyManager;
pub use cargo_manager::CargoManager;
pub use homebrew_manager::HomebrewManager;
pub use npm_manager::NpmManager;
pub use pip_manager::PipManager;

