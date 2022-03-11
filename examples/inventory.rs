// Show Desktop App list example (using WinRT "Windows.Inventory.InstalledDesktopApp")

use windows::core::{
    Result,
};

/* required feature "System_Inventory" */
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
    env_logger::init();

    if let Err(e) = windows_async::block_on(show_installed_desktop_app()) {
        println!("error: {:?}", e);
    }
}
