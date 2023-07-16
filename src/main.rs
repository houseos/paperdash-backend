// Rust standard library
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::time::SystemTime;
// Image manipulation
use image::io::Reader;
use image::{Pixel, RgbImage};
// Webserver & webencoding
#[macro_use]
extern crate rocket;
use rocket::response::stream::ByteStream;
use urlencoding::encode;
// Our modules
mod modules;
use modules::config::{
    get_website_ip, get_website_port, get_widgets_of_device, init_config, Widget,
};
use modules::screenshot::take_screenshot;

/// Enum containing colors supported by e-paper display
enum ImageColor {
    BLACK,
    RED,
}
// black&white image routes

#[get("/s2/bw?<id>")]
fn s2_bw(id: &str) -> ByteStream![Vec<u8>] {
    screenshot_paperdash_website(id);
    let bytes = get_sub_bitmap_as_vec_u8(0, 648, 0, 492, ImageColor::BLACK, id);
    ByteStream! {
        yield bytes
    }
}

#[get("/m2/bw?<id>")]
fn m2_bw(id: &str) -> ByteStream![Vec<u8>] {
    screenshot_paperdash_website(id);
    let bytes = get_sub_bitmap_as_vec_u8(648, 656, 0, 492, ImageColor::BLACK, id);
    ByteStream! {
        yield bytes
    }
}
#[get("/m1/bw?<id>")]
fn m1_bw(id: &str) -> ByteStream![Vec<u8>] {
    screenshot_paperdash_website(id);
    let bytes = get_sub_bitmap_as_vec_u8(0, 648, 492, 492, ImageColor::BLACK, id);
    ByteStream! {
        yield bytes
    }
}
#[get("/s1/bw?<id>")]
fn s1_bw(id: &str) -> ByteStream![Vec<u8>] {
    screenshot_paperdash_website(id);
    let bytes = get_sub_bitmap_as_vec_u8(648, 656, 492, 492, ImageColor::BLACK, id);
    ByteStream! {
        yield bytes
    }
}

// red image routes

#[get("/s2/r?<id>")]
fn s2_r(id: &str) -> ByteStream![Vec<u8>] {
    screenshot_paperdash_website(id);
    let bytes = get_sub_bitmap_as_vec_u8(0, 648, 0, 492, ImageColor::RED, id);
    ByteStream! {
        yield bytes
    }
}

#[get("/m2/r?<id>")]
fn m2_r(id: &str) -> ByteStream![Vec<u8>] {
    screenshot_paperdash_website(id);
    let bytes = get_sub_bitmap_as_vec_u8(648, 656, 0, 492, ImageColor::RED, id);
    ByteStream! {
        yield bytes
    }
}
#[get("/m1/r?<id>")]
fn m1_r(id: &str) -> ByteStream![Vec<u8>] {
    screenshot_paperdash_website(id);
    let bytes = get_sub_bitmap_as_vec_u8(0, 648, 492, 492, ImageColor::RED, id);
    ByteStream! {
        yield bytes
    }
}
#[get("/s1/r?<id>")]
fn s1_r(id: &str) -> ByteStream![Vec<u8>] {
    screenshot_paperdash_website(id);
    let bytes = get_sub_bitmap_as_vec_u8(648, 656, 492, 492, ImageColor::RED, id);
    ByteStream! {
        yield bytes
    }
}

/// Retrieve a Vec<u8> that contains the pixels of an image
///
/// Each bit in a u8 represents one pixel.
/// *_offset and *_len can be used to return the enclosed part of the image.
/// color defines whether the returned Vec<u8> should contain black or red pixels.
/// id is used to load the correct input image from the cache directory.
fn get_sub_bitmap_as_vec_u8(
    x_offset: u32,
    x_len: u32,
    y_offset: u32,
    y_len: u32,
    color: ImageColor,
    id: &str,
) -> Vec<u8> {
    // Assemble filepath
    let filename = format!("cache/img_{}.png", id.replace(':', "_"));
    // Open file, decode and transform it into RGB8
    let img: RgbImage = Reader::open(filename)
        .unwrap()
        .decode()
        .unwrap()
        .into_rgb8();
    // New Vec<u8> for pixel bytes
    let mut vec: Vec<u8> = Vec::new();
    // For each row in the image, starting at y_offset until y_offset+y_len is reached
    for y in y_offset..(y_offset + y_len) {
        // For each pixel, starting with x_offset until x_offset+x_len is reached, in 8-pixel steps
        for x in (x_offset..(x_offset + x_len)).step_by(8) {
            let mut byte: u8 = 0;
            // Depending on the provided color, assemble the byte containing 8 pixels
            match color {
                ImageColor::BLACK => {
                    byte = byte | (get_mono_pixel_as_u8(&img, x, y) << 7);
                    byte = byte | (get_mono_pixel_as_u8(&img, x + 1, y) << 6);
                    byte = byte | (get_mono_pixel_as_u8(&img, x + 2, y) << 5);
                    byte = byte | (get_mono_pixel_as_u8(&img, x + 3, y) << 4);
                    byte = byte | (get_mono_pixel_as_u8(&img, x + 4, y) << 3);
                    byte = byte | (get_mono_pixel_as_u8(&img, x + 5, y) << 2);
                    byte = byte | (get_mono_pixel_as_u8(&img, x + 6, y) << 1);
                    byte = byte | (get_mono_pixel_as_u8(&img, x + 7, y));
                }
                ImageColor::RED => {
                    byte = byte | (get_red_pixel_as_u8(&img, x, y) << 7);
                    byte = byte | (get_red_pixel_as_u8(&img, x + 1, y) << 6);
                    byte = byte | (get_red_pixel_as_u8(&img, x + 2, y) << 5);
                    byte = byte | (get_red_pixel_as_u8(&img, x + 3, y) << 4);
                    byte = byte | (get_red_pixel_as_u8(&img, x + 4, y) << 3);
                    byte = byte | (get_red_pixel_as_u8(&img, x + 5, y) << 2);
                    byte = byte | (get_red_pixel_as_u8(&img, x + 6, y) << 1);
                    byte = byte | (get_red_pixel_as_u8(&img, x + 7, y));
                }
            }
            // Add byte to Vec<u8>
            vec.push(byte);
        }
    }
    // Return Vec<u8>
    vec
}

fn get_red_pixel_as_u8(img: &RgbImage, x: u32, y: u32) -> u8 {
    let rgb = img.get_pixel(x, y).to_rgb();
    let r: u8 = rgb[0];
    let g = rgb[1];
    let b = rgb[2];
    if r >= 128 && g < 128 && b < 128 {
        1
    } else {
        0
    }
}

fn get_mono_pixel_as_u8(img: &RgbImage, x: u32, y: u32) -> u8 {
    let rgb = img.get_pixel(x, y).to_rgb();
    let r: u8 = rgb[0];
    let g: u8 = rgb[1];
    let b: u8 = rgb[2];
    if r > 10 || g > 10 || b > 10 {
        // Black
        0
    } else {
        // White
        1
    }
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    init_config();
    let _rocket = rocket::build()
        .mount(
            "/paperdash",
            routes![s2_bw, m2_bw, m1_bw, s1_bw, s2_r, m2_r, m1_r, s1_r],
        )
        .launch()
        .await
        .unwrap();

    Ok(())
}

fn screenshot_paperdash_website(id: &str) {
    let widgets: Vec<Widget> = get_widgets_of_device(id);

    let filename: String = format!("cache/img_{}.png", id.replace(':', "_"));
    if Path::new(&filename).exists() {
        // check if file is older than 1min
        let mut older = false;
        let metadata = fs::metadata(&filename).unwrap();
        if let Ok(time) = metadata.modified() {
            let now = SystemTime::now();
            let difference = now.duration_since(time).unwrap();
            if difference.as_secs() > 60 {
                older = true;
            }
        }
        // if file is older, take new screenshot
        if older {
            screenshot(filename, widgets);
        }
    } else {
        screenshot(filename, widgets);
    }
}

fn screenshot(filename: String, widgets: Vec<Widget>) {
    let web_img = take_screenshot(
        format!(
            "http://{}:{}/?widgets={}",
            get_website_ip(),
            get_website_port(),
            encode(&serde_json::to_string(&widgets).unwrap())
        )
        .to_string(),
        1304,
        984,
    );
    let mut file = File::create(filename).unwrap();
    file.write_all(web_img.as_slice()).unwrap();
}
