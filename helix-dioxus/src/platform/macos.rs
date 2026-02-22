use objc2::{AnyThread, MainThreadMarker};
use objc2_app_kit::{NSApplication, NSImage};
use objc2_foundation::NSData;

/// Set the macOS dock icon from PNG bytes at runtime.
///
/// This is needed because `cargo install` / `cargo run` produces a bare binary
/// without a `.app` bundle, so macOS has no `Info.plist` to find the icon.
/// Calling `NSApplication::setApplicationIconImage` sets it programmatically.
#[allow(unsafe_code)]
pub fn set_dock_icon(icon_bytes: &[u8]) {
    // SAFETY: This function is called from the Dioxus event handler closure,
    // which runs on the main thread.
    let mtm = unsafe { MainThreadMarker::new_unchecked() };

    let data = NSData::with_bytes(icon_bytes);
    let Some(image) = NSImage::initWithData(NSImage::alloc(), &data) else {
        log::warn!("Failed to create NSImage from icon bytes");
        return;
    };

    let app = NSApplication::sharedApplication(mtm);
    // SAFETY: Passing a valid NSImage is always allowed.
    unsafe { app.setApplicationIconImage(Some(&image)) };
}
