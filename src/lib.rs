extern crate image;

use std::fs::File;
use std::path::Path;

use image::Pixel;

pub mod pixel_filters;

pub enum PixelFilters {
    None,
    Inverted,
    RedAverage,
    RedWeightedLow,
    RedWeightedMid,
    RedWeightedHigh,
    RedWeightedCustom,
}


pub fn apply_filter(input_name: &str, filter: PixelFilters, option: f32) -> image::DynamicImage {
    let mut img = image::open(&Path::new(input_name)).unwrap();

    for pixel in img.as_mut_rgb8().unwrap().pixels_mut() {
        match filter {
            PixelFilters::None => {}
            PixelFilters::Inverted => {
                pixel.invert();
            }
            PixelFilters::RedAverage => {
                *pixel = image::Rgb(pixel_filters::red_averages(pixel.channels()));
            }
            PixelFilters::RedWeightedLow => {
                *pixel = image::Rgb(pixel_filters::red_weighted(pixel.channels(), 1.1f32));
            }
            PixelFilters::RedWeightedMid => {
                *pixel = image::Rgb(pixel_filters::red_weighted(pixel.channels(), 1.9f32));
            }
            PixelFilters::RedWeightedHigh => {
                *pixel = image::Rgb(pixel_filters::red_weighted(pixel.channels(), 2.8f32));
            }
            PixelFilters::RedWeightedCustom => {
                *pixel = image::Rgb(pixel_filters::red_weighted(pixel.channels(), option));
            }
        }
    }
    return img;
}

pub fn save_to_jpg_file(img : image::DynamicImage, name : &str) {
    let ref mut fout = File::create(&Path::new(name)).unwrap();
    let _    = img.save(fout, image::JPEG);
}
