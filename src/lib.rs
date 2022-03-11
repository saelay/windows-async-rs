//! Simple async executor for windows application using
//! [windows](https://github.com/microsoft/windows-rs) crate.
//! 
//! # Examples
//! ```
//! // Show Desktop App list example (using WinRT "Windows.Inventory.InstalledDesktopApp")
//! 
//! use windows::core::{
//!     Result,
//! };
//! 
//! use windows::System::Inventory::{
//!     InstalledDesktopApp,
//! };
//! 
//! async fn show_installed_desktop_app() -> Result<()> {
//! 
//!     let vec = InstalledDesktopApp::GetInventoryAsync()?.await?;
//! 
//!     for i in 0..vec.Size()? {
//!         let item = vec.GetAt(i)?;
//!         println!("Id: {:?}", item.Id()?);
//!         println!("DisplayName: {:?}", item.DisplayName()?);
//!         println!("Publisher: {:?}", item.Publisher()?);
//!         println!("DisplayVersion: {:?}", item.DisplayVersion()?);
//!         println!();
//!     }
//! 
//!     Ok(())
//! }
//! 
//! fn main() {
//!     if let Err(e) = windows_async::block_on(show_installed_desktop_app()) {
//!         println!("error: {:?}", e);
//!     }
//! }
//! ```
//! 

mod executor;

pub use executor::{
    block_on,
    create_dummy_window,
    DummyWindow,
};
