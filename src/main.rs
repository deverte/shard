//! Shard is a CLI program for offscreen rendering WGSL pixel (fragment) shaders
//! into images (PNG) or animations (GIF).
//!
//! Author: Artem Shepelin <4.shepelin@gmail.com>
//! License: GPL-3

use clap::Parser;


mod animation;
mod image;


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(help = "Input WGSL (.wgsl) fragment (pixel) shader")]
    input: std::path::PathBuf,

    #[
        arg(
            short,
            long,
            default_value = "image.png",
            help = "Output file (with .png or .gif extension)",
        )
    ]
    output: std::path::PathBuf,

    #[
        arg(
            short,
            long = "width",
            value_name = "WIDTH",
            default_value_t = 512,
            help = "Image width (must be multiple of [256 / 4])",
        )
    ]
    x: usize,

    #[
        arg(
            short,
            long = "height",
            value_name = "HEIGHT",
            default_value_t = 512,
            help = "Image height",
        )
    ]
    y: usize,

    #[
        arg(
            short,
            long,
            default_value_t = 60,
            help = "Number of frames (for GIF output)",
        )
    ]
    frames_count: usize,
}


pub fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp_nanos()
        .init();

    let args = Args::parse();

    match args.output.extension().unwrap().to_str() {
        Some("png") => {
            pollster::block_on(
                image::read_and_save(
                    args.input,
                    args.output,
                    (args.x, args.y),
                )
            );
        },
        Some("gif") => {
            pollster::block_on(
                animation::read_and_save(
                    args.input,
                    args.output,
                    (args.x, args.y),
                    args.frames_count,
                ),
            );
        },
        _ => println!("Unknown format."),
    }
}