use std::fs::File;
use std::io::Write;
use std::thread;
use std::time::Duration;

use image::io::Reader;
use image::{DynamicImage, GenericImageView, ImageBuffer, ImageFormat, Rgb, RgbImage};
use rocket::response::stream::ByteStream;
#[macro_use]
extern crate rocket;
use bmp::Image;
use webscreenshotlib::{screenshot_tab, OutputFormat};
#[get("/s2")]
fn s2() -> ByteStream![Vec<u8>] {
    ByteStream! {
        yield get_sub_bitmap_as_vec_u8(0,648,0,492)
    }
}

#[get("/m2")]
fn m2() -> ByteStream![Vec<u8>] {
    ByteStream! {
        yield get_sub_bitmap_as_vec_u8(648,656,0,492)
    }
}
#[get("/m1")]
fn m1() -> ByteStream![Vec<u8>] {
    ByteStream! {
        yield get_sub_bitmap_as_vec_u8(0,648,492,492)
    }
}
#[get("/s1")]
fn s1() -> ByteStream![Vec<u8>] {
    ByteStream! {
        yield get_sub_bitmap_as_vec_u8(648,656,492,492)
    }
}

fn get_sub_bitmap_as_vec_u8(x_offset: u32, x_len: u32, y_offset: u32, y_len: u32) -> Vec<u8> {
    /*let img = bmp::open("../example.bmp").unwrap_or_else(|e| {
        panic!("Failed to open: {}", e);
    });*/

    let img: RgbImage = Reader::open("img.png")
        .unwrap()
        .decode()
        .unwrap()
        .into_rgb8();
    let mut vec: Vec<u8> = Vec::new();
    for y in y_offset..(y_offset + y_len) {
        for x in (x_offset..(x_offset + x_len)).step_by(8) {
            let mut byte: u8 = 0;
            byte = byte | (get_mono_pixel_as_u8(&img, x, y) << 7);
            byte = byte | (get_mono_pixel_as_u8(&img, x + 1, y) << 6);
            byte = byte | (get_mono_pixel_as_u8(&img, x + 2, y) << 5);
            byte = byte | (get_mono_pixel_as_u8(&img, x + 3, y) << 4);
            byte = byte | (get_mono_pixel_as_u8(&img, x + 4, y) << 3);
            byte = byte | (get_mono_pixel_as_u8(&img, x + 5, y) << 2);
            byte = byte | (get_mono_pixel_as_u8(&img, x + 6, y) << 1);
            byte = byte | (get_mono_pixel_as_u8(&img, x + 7, y));
            vec.push(byte);
        }
    }
    vec
}

fn get_mono_pixel_as_u8(img: &RgbImage, x: u32, y: u32) -> u8 {
    if img.get_pixel(x, y)[0] >= 1 {
        1
    } else {
        0
    }
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let web_img = take_screenshot("http://192.168.2.165:5173".to_string(), 1304, 984);
    let mut file = File::create("img.png").unwrap();
    file.write_all(web_img.as_slice()).unwrap();

    let _rocket = rocket::build()
        .mount("/paperdash", routes![s2, m2, m1, s1])
        .launch()
        .await?;

    Ok(())
}

fn take_screenshot(url: String, width: u16, height: u16) -> Vec<u8> {
    let image_data = screenshot_tab(&url, OutputFormat::PNG, 80, false, width, height, "").unwrap();

    image_data
}
