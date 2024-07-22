use std::{thread, time::Duration};

use headless_chrome::{
    protocol::cdp::{Page, Target::CreateTarget},
    Browser,
};

pub fn take_screenshot(url: String, width: u32, height: u32) -> Vec<u8> {
    let launch_options = LaunchOptions::default();
    let browser = Browser::new(launch_options)?;
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

    image_data
}
