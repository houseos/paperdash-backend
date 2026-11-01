use std::{ffi::OsStr, path::PathBuf, thread, time::Duration};

use headless_chrome::{
    protocol::cdp::{Emulation, Page, Target::CreateTarget},
    Browser, LaunchOptions,
};

pub fn take_screenshot(url: String, width: u32, height: u32) -> Result<Vec<u8>, String> {
    println!("Launching browser");
    let mut launch_options = LaunchOptions::default();
    launch_options.enable_logging = true;
    launch_options.sandbox = false;
    if let Some(path) = std::env::var_os("CHROME") {
        launch_options.path = Some(PathBuf::from(path));
    }
    launch_options
        .args
        .push(OsStr::new("--disable-web-security"));
    let browser = Browser::new(launch_options).map_err(|error| error.to_string())?;
    println!("Browser launched, opening url: {}", url);
    let tab = browser
        .new_tab_with_options(CreateTarget {
            url,
            left: Some(0),
            top: Some(0),
            width: Some(width),
            height: Some(height),
            window_state: None,
            browser_context_id: None,
            enable_begin_frame_control: None,
            new_window: Some(true),
            background: Some(false),
            for_tab: None,
            hidden: None,
        })
        .map_err(|error| error.to_string())?;

    tab.wait_until_navigated()
        .map_err(|error| error.to_string())?;
    tab.call_method(Emulation::SetDeviceMetricsOverride {
        width,
        height,
        device_scale_factor: 1.0,
        mobile: false,
        scale: None,
        screen_width: Some(width),
        screen_height: Some(height),
        position_x: None,
        position_y: None,
        dont_set_visible_size: None,
        screen_orientation: None,
        viewport: None,
        display_feature: None,
        device_posture: None,
    })
    .map_err(|error| error.to_string())?;
    println!("tab opened");
    thread::sleep(Duration::from_secs(2));

    // Take a screenshot of just the WebKit-Infobox
    let image_data = tab
        .capture_screenshot(
            Page::CaptureScreenshotFormatOption::Png,
            Some(80),
            None,
            true,
        )
        .map_err(|error| error.to_string())?;
    let image = image::load_from_memory(&image_data).map_err(|error| error.to_string())?;
    if image.width() != width || image.height() != height {
        return Err(format!(
            "Screenshot size was {}x{}, expected {width}x{height}",
            image.width(),
            image.height()
        ));
    }
    println!("screenshot taken");
    Ok(image_data)
}
