extern crate image;
extern crate imageproc;
extern crate palette;
extern crate rand;
extern crate rayon;

use image::{ImageBuffer, Luma, RgbImage};
use imageproc::drawing::draw_line_segment_mut;
use palette::{Gradient, Lab, LinSrgb};
use rand::distributions::{IndependentSample, Normal};
use rayon::prelude::*;
use std::env;
use std::time::Instant;

type Image = ImageBuffer<Luma<f32>, Vec<f32>>;

fn run_series(series: &[u32], width: u32, height: u32) -> Image {
    // initialize new image
    let mut data = Image::new(width, height);

    // draw the time series as a line
    for x in 0..series.len() - 1 {
        draw_line_segment_mut(
            &mut data,
            (x as f32, series[x] as f32),
            ((x + 1) as f32, series[x + 1] as f32),
            Luma([1.0]),
        );
    }

    // normalize
    for x in 0..width {
        let mut sum = 0.0;
        for y in 0..height {
            sum += data.get_pixel(x, y)[0];
        }
        for y in 0..height {
            let value = data.get_pixel(x, y)[0];
            data.put_pixel(x, y, Luma([value / sum]));
        }
    }

    data
}

/// Reducer that combines counts from two time series.
fn sum_images(image: Image, mut aggregated: Image) -> Image {
    for (x, y, value) in image.enumerate_pixels() {
        let new_value = aggregated.get_pixel(x, y)[0] + value[0];
        aggregated.put_pixel(x, y, Luma([new_value]))
    }

    aggregated
}

fn main() {
    let now = Instant::now();

    let width = 400;
    let height = 300;

    // parse command line argument
    let args: Vec<_> = env::args().collect();
    let mut iterations = 100;

    if args.len() == 2 {
        iterations = match args[1].parse() {
            Ok(n) => n,
            Err(_) => {
                println!("error: argument not an integer");
                return;
            }
        };
    }

    // create sine wave as a model
    let model: Vec<f32> = (0..width)
        .map(|x| {
            let heightf = height as f32;
            let xf = x as f32;
            heightf / 4.0 * (xf / 20.0).sin() + heightf / 2.0
        })
        .collect();

    let data: Vec<Vec<u32>> = (0..iterations)
        .map(|_| {
            // add some noise
            let normal = Normal::new(0.0, 12.0);
            let mut rng = rand::thread_rng();

            model
                .iter()
                .map(|v| {
                    let value = v + normal.ind_sample(&mut rng) as f32;
                    if value < 0.0 {
                        0u32
                    } else if value > height as f32 {
                        height
                    } else {
                        value as u32
                    }
                })
                .collect()
        })
        .collect();

    println!("Preparing data took {}s", now.elapsed().as_secs());
    let now = Instant::now();

    let aggregated = data
        .par_iter()
        .map(|series| run_series(series, width, height))
        .reduce(|| Image::new(width, height), sum_images);

    println!("Computing line density took {}s", now.elapsed().as_secs());

    // color scale to convert from value to a color
    let color_scale = Gradient::new(vec![
        Lab::from(LinSrgb::new_u8(247, 252, 241)),
        Lab::from(LinSrgb::new_u8(14, 66, 127)),
    ]);

    let mut img = RgbImage::new(width, height);

    // find the maximum value so that we can scale colors
    let max_value = aggregated
        .pixels()
        .fold(0.0, |max, pixel| f32::max(max, pixel[0]));

    // create output image
    for (x, y, pixel) in aggregated.enumerate_pixels() {
        let value = pixel[0];
        if value == 0.0 {
            img.put_pixel(x, y, image::Rgb([255, 255, 255]));
        } else {
            let color = LinSrgb::from(color_scale.get(value / max_value));
            let converted_color = image::Rgb([
                (color.red * 255.0).round() as u8,
                (color.green * 255.0).round() as u8,
                (color.blue * 255.0).round() as u8,
            ]);

            img.put_pixel(x, y, converted_color);
        }
    }

    img.save("output.png").unwrap();
}
