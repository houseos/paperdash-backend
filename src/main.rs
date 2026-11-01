use std::collections::HashMap;
// Rust standard library
use std::io::Cursor;
use std::sync::RwLock;
use std::time::SystemTime;
// Image manipulation
use image::imageops::BiLevel;
use image::ImageReader;
use image::{imageops, DynamicImage, ImageBuffer, Luma, Pixel, RgbImage};

// Webserver & webencoding
#[macro_use]
extern crate rocket;
use lazy_static::lazy_static;
use urlencoding::encode;
// Our modules
mod modules;
use modules::config::{get_website_url, get_widgets_of_device, init_config, Widget};
use modules::screenshot::take_screenshot;

// struct to store image with metadata
#[derive(Clone)]
struct CachedImage {
    timestamp: SystemTime,
    image: Vec<u8>,
}

fn cache_is_stale(cached: Option<&CachedImage>, now: SystemTime) -> bool {
    cached.is_none_or(|image| {
        now.duration_since(image.timestamp)
            .is_ok_and(|age| age > std::time::Duration::from_secs(60))
    })
}

// hashmap storing cached images
lazy_static! {
    static ref CACHE: RwLock<HashMap<String, CachedImage>> = RwLock::new(HashMap::new());
}

/// Enum containing colors supported by e-paper display
enum ImageColor {
    BLACK,
    RED,
}
// black&white image routes

#[get("/s2/bw?<id>")]
fn s2_bw(id: &str) -> Vec<u8> {
    screenshot_paperdash_website(id);
    get_sub_bitmap_as_vec_u8(0, 648, 0, 492, ImageColor::BLACK, id)
}

#[get("/m2/bw?<id>")]
fn m2_bw(id: &str) -> Vec<u8> {
    screenshot_paperdash_website(id);
    get_sub_bitmap_as_vec_u8(648, 656, 0, 492, ImageColor::BLACK, id)
}
#[get("/m1/bw?<id>")]
fn m1_bw(id: &str) -> Vec<u8> {
    screenshot_paperdash_website(id);
    get_sub_bitmap_as_vec_u8(0, 648, 492, 492, ImageColor::BLACK, id)
}
#[get("/s1/bw?<id>")]
fn s1_bw(id: &str) -> Vec<u8> {
    screenshot_paperdash_website(id);
    get_sub_bitmap_as_vec_u8(648, 656, 492, 492, ImageColor::BLACK, id)
}

// red image routes

#[get("/s2/r?<id>")]
fn s2_r(id: &str) -> Vec<u8> {
    screenshot_paperdash_website(id);
    get_sub_bitmap_as_vec_u8(0, 648, 0, 492, ImageColor::RED, id)
}

#[get("/m2/r?<id>")]
fn m2_r(id: &str) -> Vec<u8> {
    screenshot_paperdash_website(id);
    get_sub_bitmap_as_vec_u8(648, 656, 0, 492, ImageColor::RED, id)
}
#[get("/m1/r?<id>")]
fn m1_r(id: &str) -> Vec<u8> {
    screenshot_paperdash_website(id);
    get_sub_bitmap_as_vec_u8(0, 648, 492, 492, ImageColor::RED, id)
}
#[get("/s1/r?<id>")]
fn s1_r(id: &str) -> Vec<u8> {
    screenshot_paperdash_website(id);
    get_sub_bitmap_as_vec_u8(648, 656, 492, 492, ImageColor::RED, id)
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
    let Some(cached_image) = CACHE.read().unwrap().get(id).cloned() else {
        return vec![0xff; (x_len.div_ceil(8) * y_len) as usize];
    };
    let mut reader = ImageReader::new(Cursor::new(cached_image.image));
    reader.set_format(image::ImageFormat::Png);
    let Ok(image) = reader.decode() else {
        return vec![0xff; (x_len.div_ceil(8) * y_len) as usize];
    };
    let Some(x_end) = x_offset.checked_add(x_len) else {
        return vec![0xff; (x_len.div_ceil(8) * y_len) as usize];
    };
    let Some(y_end) = y_offset.checked_add(y_len) else {
        return vec![0xff; (x_len.div_ceil(8) * y_len) as usize];
    };
    if x_end > image.width() || y_end > image.height() {
        return vec![0xff; (x_len.div_ceil(8) * y_len) as usize];
    }
    let img = match color {
        ImageColor::BLACK => dither(image.to_luma8()).into_rgb8(),
        ImageColor::RED => image.into_rgb8(),
    };

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

fn dither(mut input: ImageBuffer<Luma<u8>, Vec<u8>>) -> DynamicImage {
    let cmap = BiLevel;
    imageops::dither(&mut input, &cmap);
    DynamicImage::ImageLuma8(input.clone())
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
    if r > 250 || g > 250 || b > 250 {
        // Black
        1
    } else {
        // White
        0
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

    {
        if !CACHE.read().unwrap().contains_key(id) {
            screenshot(id, widgets);
            return;
        }
    }
    let stale = cache_is_stale(CACHE.read().unwrap().get(id), SystemTime::now());
    if stale {
        screenshot(id, widgets);
    }
}

fn screenshot(id: &str, widgets: Vec<Widget>) {
    let web_img = match take_screenshot(
        format!(
            "{}/?widgets={}",
            get_website_url(),
            encode(&serde_json::to_string(&widgets).unwrap())
        )
        .to_string(),
        1304,
        984,
    ) {
        Ok(image) => image,
        Err(error) => {
            eprintln!("Could not capture dashboard for {id}: {error}");
            return;
        }
    };
    let mut c = CACHE.write().unwrap();
    c.insert(
        id.to_string(),
        CachedImage {
            timestamp: SystemTime::now(),
            image: web_img,
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageFormat, Rgb};

    fn cache_test_image(id: &str, image: DynamicImage) {
        let mut png = Cursor::new(Vec::new());
        image.write_to(&mut png, ImageFormat::Png).unwrap();
        CACHE.write().unwrap().insert(
            id.to_string(),
            CachedImage {
                timestamp: SystemTime::now(),
                image: png.into_inner(),
            },
        );
    }

    #[test]
    fn packs_white_and_black_pixels_most_significant_bit_first() {
        let id = "test-monochrome-packing";
        let image = ImageBuffer::from_fn(8, 1, |x, _| {
            if x < 4 {
                Rgb([255, 255, 255])
            } else {
                Rgb([0, 0, 0])
            }
        });
        cache_test_image(id, DynamicImage::ImageRgb8(image));

        let bytes = get_sub_bitmap_as_vec_u8(0, 8, 0, 1, ImageColor::BLACK, id);

        assert_eq!(bytes, [0xf0]);
        CACHE.write().unwrap().remove(id);
    }

    #[test]
    fn red_plane_marks_only_red_pixels() {
        let id = "test-red-plane";
        let image = ImageBuffer::from_fn(8, 1, |x, _| match x {
            0 => Rgb([255, 0, 0]),
            1 => Rgb([200, 20, 10]),
            2 => Rgb([255, 0, 255]),
            3 => Rgb([255, 100, 0]),
            _ => Rgb([0, 0, 0]),
        });
        cache_test_image(id, DynamicImage::ImageRgb8(image));

        let bytes = get_sub_bitmap_as_vec_u8(0, 8, 0, 1, ImageColor::RED, id);

        assert_eq!(bytes, [0xd0]);
        CACHE.write().unwrap().remove(id);
    }

    #[test]
    fn encodes_all_four_panel_quadrants_at_controller_widths() {
        let id = "test-panel-quadrants";
        let image = ImageBuffer::from_fn(1304, 984, |x, y| {
            if (x < 648 && y < 492) || (x >= 648 && y >= 492) {
                Rgb([255, 0, 0])
            } else {
                Rgb([0, 0, 0])
            }
        });
        cache_test_image(id, DynamicImage::ImageRgb8(image));

        let s2 = get_sub_bitmap_as_vec_u8(0, 648, 0, 492, ImageColor::RED, id);
        let m2 = get_sub_bitmap_as_vec_u8(648, 656, 0, 492, ImageColor::RED, id);
        let m1 = get_sub_bitmap_as_vec_u8(0, 648, 492, 492, ImageColor::RED, id);
        let s1 = get_sub_bitmap_as_vec_u8(648, 656, 492, 492, ImageColor::RED, id);

        assert_eq!(s2.len(), 81 * 492);
        assert_eq!(m2.len(), 82 * 492);
        assert_eq!(m1.len(), 81 * 492);
        assert_eq!(s1.len(), 82 * 492);
        assert_eq!(s2[0], 0xff);
        assert_eq!(m2[0], 0x00);
        assert_eq!(m1[0], 0x00);
        assert_eq!(s1[0], 0xff);
        CACHE.write().unwrap().remove(id);
    }

    #[test]
    fn missing_or_invalid_cached_png_returns_white_pixels() {
        let missing = "test-missing-cache-entry";
        let invalid = "test-invalid-cache-entry";
        CACHE.write().unwrap().insert(
            invalid.to_string(),
            CachedImage {
                timestamp: SystemTime::now(),
                image: b"not a png".to_vec(),
            },
        );

        assert_eq!(
            get_sub_bitmap_as_vec_u8(0, 16, 0, 2, ImageColor::RED, missing),
            [0xff; 4]
        );
        assert_eq!(
            get_sub_bitmap_as_vec_u8(0, 16, 0, 2, ImageColor::BLACK, invalid),
            [0xff; 4]
        );
        let mut cache = CACHE.write().unwrap();
        cache.remove(invalid);
        cache.remove(missing);
    }

    #[test]
    fn undersized_png_returns_white_pixels_instead_of_panicking() {
        let id = "test-undersized-cache-entry";
        cache_test_image(
            id,
            DynamicImage::ImageRgb8(ImageBuffer::from_pixel(7, 1, Rgb([0, 0, 0]))),
        );

        assert_eq!(
            get_sub_bitmap_as_vec_u8(0, 8, 0, 1, ImageColor::BLACK, id),
            [0xff]
        );
        CACHE.write().unwrap().remove(id);
    }

    #[test]
    fn screenshot_cache_expires_after_one_minute() {
        let now = SystemTime::now();
        let fresh = CachedImage {
            timestamp: now - std::time::Duration::from_secs(60),
            image: Vec::new(),
        };
        let just_expired = CachedImage {
            timestamp: now - std::time::Duration::from_millis(60_001),
            image: Vec::new(),
        };

        assert!(cache_is_stale(None, now));
        assert!(!cache_is_stale(Some(&fresh), now));
        assert!(cache_is_stale(Some(&just_expired), now));
    }
}
