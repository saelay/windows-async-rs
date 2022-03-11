// OCR example (using WinRT "Windows.Media.Ocr.OcrEngine")

use windows::core::{
    Result,
    HSTRING,
    Interface as _,
};

/* required feature "Win32_UI_Shell" */
use windows::Win32::UI::Shell::{
    IInitializeWithWindow,
};

/* required feature "Graphics_Imaging" */
use windows::Graphics::Imaging::{
    BitmapDecoder,
};

/* required feature "Media_Ocr" */
use windows::Media::Ocr::{
    OcrEngine,
};

/* required feature "Storage" */
use windows::Storage::{
    FileAccessMode,
};

/* required feature "Storage_Pickers" */
use windows::Storage::Pickers::{
    FileOpenPicker,
};


async fn exec_ocr() -> Result<String> {

    let picker = FileOpenPicker::new()?;
    picker.FileTypeFilter()?.Append(HSTRING::from(".png"))?;
    
    // FileOpenPicker requires parent HWND...
    // https://devblogs.microsoft.com/oldnewthing/20190412-00/?p=102413
    let file = {
        let dummy_window = windows_async::create_dummy_window();
        unsafe { picker.cast::<IInitializeWithWindow>()?.Initialize(dummy_window.hwnd())?; }

        picker.PickSingleFileAsync()?.await?
    };

    let bmp = BitmapDecoder::CreateWithIdAsync(
        BitmapDecoder::PngDecoderId()?,
        file.OpenAsync(FileAccessMode::Read)?.await?
    )?.await?;
    let bmp = bmp.GetSoftwareBitmapAsync()?.await?;

    let ocr:OcrEngine = OcrEngine::TryCreateFromUserProfileLanguages()?;
    let ocr_result = ocr.RecognizeAsync(bmp)?.await?;

    let text = String::from_utf16_lossy(ocr_result.Text()?.as_wide());
    Ok(text)
}

fn main() {
    env_logger::init();

    match windows_async::block_on(exec_ocr()) {
        Ok(s) => {
            println!("OCR text: {}", s);
        },
        Err(e) => {
            println!("error: {:?}", e);
        }
    }
}
