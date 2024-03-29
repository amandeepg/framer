use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
/// Framer will composite a screenshot onto a device frame, by intelligently finding the space for the screenshot.
pub(crate) struct Args {
    /// Path to device frame image.
    #[arg()]
    pub(crate) device_frame_path: PathBuf,

    /// Path to screenshot image.
    #[arg()]
    pub(crate) screenshot_path: PathBuf,

    /// Path to composited output image.
    #[arg(short, default_value = "result.png")]
    pub(crate) output_path: PathBuf,

    /// How far, as a percentage, from the left edge that the screenshot area is at.
    #[arg(short, long, default_value_t = 50, value_name = "percent", value_parser = clap::value_parser ! (u8).range(0..=100))]
    pub(crate) x_perc: u8,

    /// How far, as a percentage, from the top edge that the screenshot area is at.
    #[arg(short, long, default_value_t = 50, value_name = "percent", value_parser = clap::value_parser ! (u8).range(0..=100))]
    pub(crate) y_perc: u8,

    /// The level of optimization to use with oxipng (0-6), lower is faster.
    #[arg(long, value_parser = clap::value_parser ! (u8).range(0..=6), default_value_t = 4, value_name = "level")]
    pub(crate) oxipng_level: u8,

    /// The level of optimization to use with pngquant (1-10), higher is faster.
    #[arg(long, value_parser = clap::value_parser ! (u8).range(1..=10), default_value_t = 4, value_name = "speed")]
    pub(crate) pngquant_speed: u8,
}
