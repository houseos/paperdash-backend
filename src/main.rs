use rocket::response::stream::ByteStream;
#[macro_use]
extern crate rocket;
use bmp::Image;
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
    let img = bmp::open("../example.bmp").unwrap_or_else(|e| {
        panic!("Failed to open: {}", e);
    });
    let mut vec: Vec<u8> = Vec::new();
    for y in y_offset..(y_offset + y_len) {
        for x in (x_offset..(x_offset + x_len)).step_by(8) {
            let mut byte: u8 = 0;
            byte = byte | get_mono_pixel_as_u8(&img, x, y);
            byte = byte | get_mono_pixel_as_u8(&img, x + 1, y) << 1;
            byte = byte | get_mono_pixel_as_u8(&img, x + 2, y) << 2;
            byte = byte | get_mono_pixel_as_u8(&img, x + 3, y) << 3;
            byte = byte | get_mono_pixel_as_u8(&img, x + 4, y) << 4;
            byte = byte | get_mono_pixel_as_u8(&img, x + 5, y) << 5;
            byte = byte | get_mono_pixel_as_u8(&img, x + 6, y) << 6;
            byte = byte | get_mono_pixel_as_u8(&img, x + 7, y) << 7;
            vec.push(byte);
        }
    }
    vec
}

fn get_mono_pixel_as_u8(img: &Image, x: u32, y: u32) -> u8 {
    if img.get_pixel(x, y).r >= 1 {
        1
    } else {
        0
    }
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let _rocket = rocket::build()
        .mount("/paperdash", routes![s2, m2, m1, s1])
        .launch()
        .await?;

    Ok(())
}
