# windows-async-rs

Simple async executor for windows application using windows crate.

## Example

``` Rust
// Show Desktop App list example (using WinRT "Windows.Inventory.InstalledDesktopApp")
use windows::core::{
    Result,
};

use windows::System::Inventory::{
    InstalledDesktopApp,
};

async fn show_installed_desktop_app() -> Result<()> {

    let vec = InstalledDesktopApp::GetInventoryAsync()?.await?;

    for i in 0..vec.Size()? {
        let item = vec.GetAt(i)?;
        println!("Id: {:?}", item.Id()?);
        println!("DisplayName: {:?}", item.DisplayName()?);
        println!("Publisher: {:?}", item.Publisher()?);
        println!("DisplayVersion: {:?}", item.DisplayVersion()?);
        println!();
    }

    Ok(())
}

fn main() {
    if let Err(e) = windows_async::block_on(show_installed_desktop_app()) {
        println!("error: {:?}", e);
    }
}
```