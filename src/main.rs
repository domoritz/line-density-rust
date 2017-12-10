extern crate image;
extern crate imageproc;
extern crate rand;
extern crate palette;

use image::{Luma, ImageBuffer, RgbImage};
use imageproc::drawing::{draw_line_segment_mut, draw_filled_rect_mut};
use imageproc::rect::Rect;
use rand::distributions::{IndependentSample, Range};
use palette::{Lab, Rgb, Gradient};

/// Returns a vector of length `size` with numbers in the range `[0, maximum)`.
fn vector(size: u32, maximum: u32) -> Vec<u32> {
    let mut zero_vec: Vec<u32> = Vec::with_capacity(size as usize);
    let between = Range::new(0, maximum);
    let mut rng = rand::thread_rng();

    for _ in 0..size {
        zero_vec.push(between.ind_sample(&mut rng));
    }
    
    return zero_vec;
}

type Image = ImageBuffer<Luma<f32>, Vec<f32>>;


fn main() {
    let width = 300;
    let height = 200;

    let series = vector(width, height);

    let mut data = Image::new(width, height);
    let mut aggregated = Image::new(width, height);

    for _ in 0..50000 {
        // draw the time series as a line
        for x in 0..series.len() - 1 {
            draw_line_segment_mut(
                &mut data,
                (x as f32, series[x] as f32),
                ((x + 1) as f32, series[x + 1]  as f32),
                Luma([1.0]),
            );
        }

        // normalize
        for x in 0..width {
            let mut sum = 0.0;
            for y in 0..height {
                sum += data.get_pixel(x,y).data[0];
            }
            for y in 0..height {
                let value = data.get_pixel(x,y).data[0];
                data.put_pixel(x,y,Luma([value / sum]));
            }
        }

        // add to one matrix with the sum
        for (x,y,value) in data.enumerate_pixels() {
            let new_value = aggregated.get_pixel(x,y).data[0] + value.data[0];
            aggregated.put_pixel(x,y,Luma([new_value]))
        }

        // reset the canvas
        draw_filled_rect_mut(
            &mut data,
            Rect::at(0, 0).of_size(width, height),
            Luma([0.0]),
        );
    }

    // color scale to convert from value to a color
    let color_scale = Gradient::new(vec![
        Lab::from(Rgb::new_u8(247, 252, 241)),
        Lab::from(Rgb::new_u8(14, 66, 127))
    ]);

    let mut img = RgbImage::new(width, height);

    // find the maximum value so that we can scale colors
    let max_value = aggregated.pixels().fold(
        0./0.,
        |max,pixel| f32::max(max, pixel.data[0])
    );

    // create output image
    for (x,y,pixel) in aggregated.enumerate_pixels() {
        let value = pixel.data[0];
        if value == 0.0 {
            img.put_pixel(x,y,image::Rgb([255,255,255]));
        } else {
            let color = Rgb::from(color_scale.get(value / max_value));
            let converted_color = image::Rgb([
                (color.red * 255.0).round() as u8,
                (color.green * 255.0).round() as u8,
                (color.blue * 255.0).round() as u8]
            );

            img.put_pixel(x,y,converted_color);
        }
    }

    img.save("output.png").unwrap();
}
