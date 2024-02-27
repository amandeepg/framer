mod args;
mod duration_ext;

use std::fmt::Debug;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::Instant;

use crate::args::Args;
use crate::duration_ext::DurationExt;
use anyhow::{anyhow, Context, Result};
use clap::Parser;
use colored::Colorize;
use human_bytes::human_bytes;
use image::{imageops, DynamicImage, RgbaImage};
use imageops::{crop, resize, Lanczos3};
use lodepng::Encoder;
use oxipng::{optimize_from_memory, Options};
use rgb::FromSlice;
use spinoff::{spinners, Color, Spinner};

#[derive(Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

fn search_for_non_transparent_pixel(
    img: &RgbaImage,
    dir: &Direction,
    top_search_axis: u8,
    left_search_axis: u8,
) -> Result<(u32, u32)> {
    let (width, height) = img.dimensions();
    let mut center_x = width / (100 / top_search_axis as u32);
    let mut center_y = height / (100 / left_search_axis as u32);

    while center_y > 0 && center_x > 0 && center_x < width && center_y < height {
        if img.get_pixel(center_x, center_y)[3] == 255 {
            return Ok((center_x, center_y));
        }
        match dir {
            Direction::Up => center_y -= 1,
            Direction::Down => center_y += 1,
            Direction::Left => center_x -= 1,
            Direction::Right => center_x += 1,
        }
    }

    Err(anyhow!(
        "Could not find non-transparent pixel in direction {:?}",
        dir
    ))
}

fn find_transparent_pixels(
    img: &RgbaImage,
    top_search_axis: u8,
    left_search_axis: u8,
) -> Result<(u32, u32, u32, u32)> {
    with_spinner(
        &"Finding coords of device frame ...",
        &(|res: &(u32, u32, u32, u32)| {
            Ok(format!(
                "Found coordinates of device frame: top={}px, bottom={}px, left={}px, right={}px",
                res.0, res.1, res.2, res.3,
            ))
        }),
        || {
            let directions = [
                Direction::Up,
                Direction::Down,
                Direction::Left,
                Direction::Right,
            ];
            let coords = directions
                .iter()
                .map(|dir| {
                    search_for_non_transparent_pixel(&img, dir, top_search_axis, left_search_axis)
                })
                .collect::<Result<Vec<_>>>()?;
            Ok((
                coords[0].1 - 1,
                coords[1].1 + 1,
                coords[2].0 - 1,
                coords[3].0 + 1,
            ))
        },
    )
}

fn overlay_image(
    base_img: &RgbaImage,
    overlay_png_path: &Path,
    x: u32,
    y: u32,
    target_width: u32,
    target_height: u32,
) -> Result<DynamicImage> {
    with_spinner(
        &"Overlaying images ...".to_string(),
        &(|_: &_| Ok("Overlayed images")),
        || {
            let mut base_img = base_img.clone();
            let overlay_img = image::open(overlay_png_path)?.to_rgba8();

            // resize overlay image to target size, but keep aspect ratio and crop if necessary
            let (overlay_width, overlay_height) = overlay_img.dimensions();
            let aspect_ratio = overlay_width as f32 / overlay_height as f32;
            let target_aspect_ratio = target_width as f32 / target_height as f32;

            let (overlay_width, overlay_height) = if aspect_ratio > target_aspect_ratio {
                let new_width = target_height as f32 * aspect_ratio;
                (new_width as u32, target_height)
            } else {
                let new_height = target_width as f32 / aspect_ratio;
                (target_width, new_height as u32)
            };

            let mut overlay_img = resize(&overlay_img, overlay_width, overlay_height, Lanczos3);

            overlay_img = crop(&mut overlay_img, 0, 0, target_width, target_height).to_image();

            for (x_offset, y_offset, pixel) in overlay_img.enumerate_pixels() {
                let base_pixel = base_img.get_pixel_mut(x + x_offset, y + y_offset);
                let base_alpha = base_pixel[3];
                if (base_alpha < 255) && (pixel[3] > 0) {
                    let pixel_alpha = pixel[3] as f32 / 255.0;
                    let base_alpha = base_alpha as f32 / 255.0;
                    for i in 0..3 {
                        base_pixel[i] = (base_alpha * base_pixel[i] as f32
                            + (1.0 - base_alpha) * pixel_alpha * pixel[i] as f32)
                            as u8
                    }
                    base_pixel[3] = ((base_alpha + pixel_alpha).max(1.0) * 255.0) as u8;
                }
            }

            Ok(DynamicImage::ImageRgba8(base_img))
        },
    )
}

fn oxipng_optimize(rgba: &Vec<u8>, level: u8) -> Result<Vec<u8>> {
    with_spinner(
        &format!("Optimizing image with oxipng (level {}) ...", level),
        &(|res: &Vec<u8>| {
            Ok(format!(
                "Optimized image from {} to {} ({}%) with oxipng (level {})",
                human_bytes(rgba.len() as f64),
                human_bytes(res.len() as f64),
                ((res.len() as f64 - rgba.len() as f64) / rgba.len() as f64 * 100.0).round(),
                level,
            ))
        }),
        || Ok(optimize_from_memory(&rgba, &Options::from_preset(level))?),
    )
}

fn pngquant_optimize(img: &DynamicImage, speed: u8) -> Result<Vec<u8>> {
    with_spinner(
        &format!("Optimizing image with pngquant (speed {}) ...", speed),
        &(|res: &Vec<u8>| {
            let orig_png = Encoder::new().encode(
                img.as_rgba8().context("Image not RGB*")?.as_raw(),
                img.width() as usize,
                img.height() as usize,
            )?;

            Ok(format!(
                "Optimized image from {} to {} ({}%) with pngquant (speed {})",
                human_bytes(orig_png.len() as f64),
                human_bytes(res.len() as f64),
                ((res.len() as f64 - orig_png.len() as f64) / orig_png.len() as f64 * 100.0)
                    .round(),
                speed,
            ))
        }),
        || {
            Ok({
                let new_img = img.to_rgba8();
                let new_img = new_img.as_raw();

                let mut liq = imagequant::new();
                liq.set_speed(speed as i32)?;
                let mut img = liq.new_image(
                    new_img.as_rgba(),
                    img.width() as usize,
                    img.height() as usize,
                    0.0,
                )?;
                let (palette, pixels) = liq.quantize(&mut img)?.remapped(&mut img)?;
                let mut enc = Encoder::new();
                enc.set_palette(&palette)?;
                enc.encode(&pixels, img.width(), img.height())?
            })
        },
    )
}

fn write_file(res: &Vec<u8>, path: &Path) -> Result<()> {
    with_spinner(
        &"Writing file to disk ...",
        &(|_: &()| Ok("Wrote output file to disk")),
        || {
            File::create(&path)?
                .write_all(&res)
                .map_err(anyhow::Error::from)
        },
    )
}

fn with_spinner<T, F: FnOnce() -> Result<T>, S: FnOnce(&T) -> Result<STR>, STR: Into<String>>(
    action_description: &str,
    success_func: S,
    f: F,
) -> Result<T> {
    let start = Instant::now();
    let mut spinner = Spinner::new(spinners::Dots, action_description.to_string(), Color::White);

    let result = f()?;

    let success_message = success_func(&result)?;
    spinner.success(&format!(
        "{} ({})",
        success_message.into(),
        start.elapsed().display()?
    ));

    Ok(result)
}

fn main() -> Result<()> {
    let start = Instant::now();

    let args = Args::parse();

    println!(
        "{}",
        format!(
            "Framer will place screenshot ({}) on device frame ({}) to create output ({})",
            args.screenshot_path
                .file_name()
                .context("No screenshot path")?
                .to_string_lossy(),
            args.device_frame_path
                .file_name()
                .context("No device frame path")?
                .to_string_lossy(),
            args.output_path
                .file_name()
                .context("No output path")?
                .to_string_lossy(),
        )
        .bold()
    );
    println!();

    let base_img = image::open(args.device_frame_path)?.to_rgba8();

    let (top, bottom, left, right) =
        find_transparent_pixels(&base_img, args.top_search_axis, args.left_search_axis)?;

    let overlayed_img = overlay_image(
        &base_img,
        args.screenshot_path.as_path(),
        left,
        top,
        right - left,
        bottom - top,
    )?;

    let quantized = pngquant_optimize(&overlayed_img, args.pngquant_speed)?;
    let optimized = oxipng_optimize(&quantized, args.oxipng_level)?;
    write_file(&optimized, args.output_path.as_path())?;

    println!();
    println!(
        "{} Finished! (overall: {})",
        "✔".green(),
        start.elapsed().display()?.bold()
    );
    Ok(())
}
