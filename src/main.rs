use clap::Parser;
use image::{
    codecs::gif::GifDecoder, imageops, AnimationDecoder, DynamicImage, GenericImageView,
    ImageFormat, RgbaImage,
};
use imageproc::filter::gaussian_blur_f32;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::thread;
use std::time::Duration; // Import blur function

/// Simple program to convert an image file or GIF to ASCII art
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input image file or GIF
    #[arg(short, long)]
    input: PathBuf,

    /// Width of the output ASCII art in characters
    #[arg(short, long, default_value_t = 100)]
    width: u32,

    /// Invert the character map (use for dark backgrounds)
    #[arg(long)]
    invert: bool,

    /// Adjust contrast (1.0 = normal, >1.0 = higher contrast)
    #[arg(long, default_value_t = 1.0)]
    contrast: f32,

    /// Loop GIF animation indefinitely
    #[arg(long)]
    loop_gif: bool,

    /// Output ASCII art with ANSI colors
    #[arg(long)]
    color: bool,
}

// Character sets
const ASCII_CHARS_SIMPLE: &[u8] = b" .:-=+*#%@";

const ASCII_LEN_SIMPLE: usize = ASCII_CHARS_SIMPLE.len();

// Aspect ratio correction factor (most console fonts are taller than wide)
// You might need to tweak this based on your terminal font
const ASPECT_RATIO_CORRECTION: f64 = 0.55;
const MIN_FRAME_DELAY_MS: u64 = 20; // Minimum delay for GIF frames (ms)

// Function to convert a single image frame to ASCII art with color
fn image_to_ascii(
    img: &DynamicImage,
    width: u32,
    invert: bool,
    contrast: f32,
    use_color: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    // --- Resize ---
    let new_height = (img.height() as f64 * width as f64 * ASPECT_RATIO_CORRECTION
        / img.width() as f64)
        .max(1.0) as u32; // Ensure height is at least 1
    let resized_img = img.resize_exact(width, new_height, imageops::FilterType::Lanczos3);

    // --- Map to ASCII ---
    let mut ascii_art = String::new();
    let reset_code = "\x1B[0m";

    // --- Select character set & Apply optional blur for color mode ---
    let ascii_chars = ASCII_CHARS_SIMPLE;
    let ascii_len = ASCII_LEN_SIMPLE;
    let final_img_buffer; // Need owned buffer for processing

    if use_color {
        let rgba_img = resized_img.to_rgba8();
        // Apply slight blur only for color mode to suppress noise
        final_img_buffer = DynamicImage::ImageRgba8(gaussian_blur_f32(&rgba_img, 0.6));
        ascii_art.reserve((width * new_height * 20 + new_height) as usize);
    } else {
        // No blur for grayscale
        final_img_buffer = DynamicImage::ImageLuma8(resized_img.grayscale().to_luma8());
        ascii_art.reserve((width * new_height + new_height) as usize);
    }

    // We need to handle color vs grayscale pixel access differently now
    let img_width = final_img_buffer.width();
    let img_height = final_img_buffer.height();

    for y in 0..img_height {
        for x in 0..img_width {
            let (r, g, b, luminance); // Declare vars for color/luminance

            if use_color {
                let pixel = final_img_buffer.get_pixel(x, y); // RGBA
                r = pixel[0];
                g = pixel[1];
                b = pixel[2];
                luminance = (0.2126 * r as f32) + (0.7152 * g as f32) + (0.0722 * b as f32);
                // Generate and push color code directly
                ascii_art.push_str(&format!("\x1B[38;2;{};{};{}m", r, g, b));
            } else {
                let pixel = final_img_buffer.get_pixel(x, y); // Grayscale (Luma)
                luminance = pixel[0] as f32; // Luminance is just the grayscale value
                                             // r,g,b are unused in grayscale
            }

            // --- Contrast Adjustment (on luminance/intensity) ---
            let adjusted_luminance = if contrast != 1.0 {
                let factor = contrast;
                let midpoint = 128.0;
                (factor * (luminance - midpoint) + midpoint).clamp(0.0, 255.0)
            } else {
                luminance
            };

            // --- Map Luminance/Intensity to Char Index (using selected set) ---
            let mapped_intensity = if invert {
                255.0 - adjusted_luminance as f64
            } else {
                adjusted_luminance as f64
            };
            let char_index = ((mapped_intensity / 255.0) * (ascii_len - 1) as f64).round() as usize;
            let character = ascii_chars[char_index.min(ascii_len - 1)] as char;

            // --- Append output ---
            ascii_art.push(character);
        }
        // Reset color at the end of each line if needed
        if use_color {
            ascii_art.push_str(reset_code);
        }
        ascii_art.push('\n');
    }

    Ok(ascii_art)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("Processing input: {:?}", args.input);

    // --- Detect Format ---
    let format = image::ImageFormat::from_path(&args.input)
        .map_err(|e| format!("Failed to detect image format: {}", e))?;

    match format {
        // --- Handle GIFs ---
        ImageFormat::Gif => {
            use std::io::Write; // Bring Write trait into scope for flush()

            println!("Detected GIF format. Processing frames...");
            let file_in = File::open(&args.input)?;
            let reader = BufReader::new(file_in);
            let decoder = GifDecoder::new(reader)?;
            let frames = decoder.into_frames();
            let frames = frames.collect_frames()?; // Collect frames into memory

            if frames.is_empty() {
                return Err("GIF contains no frames.".into());
            }
            println!("Processed {} frames.", frames.len());

            // --- Convert Frames to ASCII ---
            let mut ascii_frames: Vec<(String, Duration)> = Vec::with_capacity(frames.len());
            for (i, frame) in frames.iter().enumerate() {
                print!("\rConverting frame {}/{}...", i + 1, frames.len());
                // Get delay - default to 100ms if missing (common default)
                let delay = frame.delay().numer_denom_ms();
                let delay_duration = Duration::from_millis(delay.0 as u64 / delay.1 as u64);
                // Enforce minimum delay
                // let effective_delay = delay_duration.max(Duration::from_millis(MIN_FRAME_DELAY_MS));

                // Create DynamicImage from frame buffer
                let buffer: &RgbaImage = frame.buffer();
                let frame_image = DynamicImage::ImageRgba8(buffer.clone()); // Clone buffer

                // Convert frame to ASCII
                let ascii_frame = image_to_ascii(
                    &frame_image,
                    args.width,
                    args.invert,
                    args.contrast,
                    args.color,
                )?;
                // Store the *original* frame delay
                ascii_frames.push((ascii_frame, delay_duration)); // Use original delay_duration
            }
            println!("\nFrame conversion complete.");

            // --- Animate in Terminal ---
            println!("Starting animation (Press Ctrl+C to stop)...");
            // Clear screen once before starting the loop
            print!("\x1B[2J\x1B[H");
            std::io::stdout().flush()?; // Ensure clear happens now
            thread::sleep(Duration::from_millis(50)); // Small pause before starting

            loop {
                // Outer loop for optional GIF looping
                for (ascii_frame, delay) in &ascii_frames {
                    // Clear screen and move cursor to top-left (moved before loop)
                    // print!("\x1B[2J\x1B[H");
                    print!("\x1B[H"); // Move cursor to home before printing frame
                                      // Print the frame
                    print!("{}", ascii_frame);
                    // Flush stdout to ensure it's displayed immediately
                    // use std::io::Write; // Moved up
                    std::io::stdout().flush()?;
                    // Wait for the frame's effective delay (with minimum threshold)
                    // thread::sleep(*delay); // Old version
                    // Apply minimum delay threshold here
                    let effective_delay = (*delay).max(Duration::from_millis(MIN_FRAME_DELAY_MS)); // Dereference delay
                    thread::sleep(effective_delay); // Pass Duration value
                }
                if !args.loop_gif {
                    break; // Exit loop if not looping indefinitely
                }
                // println!("\rLooping...        "); // Old version
                print!("\rLooping...        "); // Use print! to avoid potential newline
                std::io::stdout().flush()?; // Flush looping message
            }
        }
        // --- Handle Static Images ---
        _ => {
            println!("Detected static image format ({:?}).", format);
            let img = image::open(&args.input)?;
            println!(
                "Image loaded successfully (Dimensions: {}x{})",
                img.width(),
                img.height()
            );

            let ascii_art =
                image_to_ascii(&img, args.width, args.invert, args.contrast, args.color)?;

            println!("\n--- Generated ASCII Art ---");
            println!("{}", ascii_art);
        }
    }

    Ok(())
}
