extern crate minifb;

use std::mem;
use std::num::Wrapping;

use image::{DynamicImage, GenericImageView};
use minifb::{Window, WindowOptions};

pub fn sum_wrap(x: u32, y: u32) -> u32 {
    (Wrapping(x) + Wrapping(y)).0 //apparently optimized to perfection
}

pub fn from_rgb(r: u8, g: u8, b:u8) -> u32 {
    from_rgba(r, g, b, 255)
}
pub fn from_rgba(r: u8, g: u8, b:u8, a:u8) -> u32 {
    unsafe {
        let a = [b, g, r, a];
        return mem::transmute::<[u8; 4], u32>(a);
    }
}

pub fn display_image_from_path(path: &str) {
    display_image(path, &image::open(path).expect("loading path failed"))
}
pub fn display_raw_image(image: &DynamicImage) {
    display_image("Image Display", image)
}
pub fn display_image(window_title: &str, image: &DynamicImage) {
    let w = image.width() as usize;
    let h = image.height() as usize;
    let mut buffer: Vec<u32> = vec![0; w * h];
    let mut index = 0;
    for p in image.pixels() {
        buffer[index] = from_rgba((p.2).0[0], (p.2).0[1], (p.2).0[2], (p.2).0[3]);
        index += 1;
    }

    let mut window = Window::new(
        window_title,
        w,
        h,
        WindowOptions::default(),
    )
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16000)));

    while window.is_open() {
        window.update_with_buffer(&buffer, w, h).expect("update window failed");
    }
}