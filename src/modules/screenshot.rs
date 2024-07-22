use std::{ffi::OsStr, thread, time::Duration};

use headless_chrome::{
    protocol::cdp::{Page, Target::CreateTarget},
    Browser, LaunchOptions,
};

pub fn take_screenshot(url: String, width: u32, height: u32) -> Vec<u8> {
    println!("Launching browser");
    let mut launch_options = LaunchOptions::default();
    launch_options.enable_logging = true;
    launch_options.sandbox = false;
    launch_options.args.push(OsStr::new("--disable-web-security"));
    let browser = Browser::new(launch_options).unwrap();
    println!("Browser launched, opening url: {}", url);
    let tab = browser
        .new_tab_with_options(CreateTarget {
            url,
            width: Some(width),
            height: Some(height),
            browser_context_id: None,
            enable_begin_frame_control: None,
            new_window: Some(false),
            background: Some(false),
        })
        .unwrap();

    tab.wait_until_navigated().unwrap();
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
        .unwrap();
    println!("screenshot taken");
    image_data
}
