//! Makes a color lookup table which is neutral.
//! This means that using the output image of this program
//! as the mapping in a LUT shader should not alter the image,
//! except for rounding errors introduced due to the size of this
//! LUT being finite.
use color_eyre::Result;
use image::{codecs::png::PngEncoder, ColorType, ImageEncoder};

const BLOCK_SIZE: u32 = 64;
const NUM_BLOCKS: u32 = 64;

const HEIGHT: u32 = BLOCK_SIZE;
const WIDTH: u32 = BLOCK_SIZE * NUM_BLOCKS;
const PIXELS: u32 = HEIGHT * WIDTH;

const COLOR_INCREMENT: f32 = 256. / (BLOCK_SIZE - 1) as f32;

#[derive(Debug, Clone, Copy)]
struct Pixel {
    r: u8,
    g: u8,
    b: u8,
}

impl From<Pixel> for Vec<u8> {
    fn from(pixel: Pixel) -> Self {
        vec![pixel.r, pixel.g, pixel.b]
    }
}

fn make_lut() -> Vec<u8> {
    let mut buf: Vec<Pixel> = Vec::with_capacity(PIXELS as usize);

    for n in 0..PIXELS {
        let block_column = n % BLOCK_SIZE;
        let block_row = n / WIDTH;
        let block_index = (n % WIDTH) / BLOCK_SIZE;

        buf.push(Pixel {
            // Red should increase from 0 to 255 within each block,
            // in the horizontal direction
            r: (block_column as f32 * COLOR_INCREMENT) as u8,

            // Green should increase from 0 to 255, vertically
            g: (block_row as f32 * COLOR_INCREMENT) as u8,

            // Blue is constant within each block
            b: (block_index as f32 * COLOR_INCREMENT) as u8,
        });
    }

    buf.into_iter().flat_map::<Vec<_>, _>(Into::into).collect()
}

fn main() -> Result<()> {
    let output_file = std::fs::File::create("lut.png")?;
    let encoder = PngEncoder::new(output_file);

    let lut = make_lut();

    encoder.write_image(&lut, WIDTH, HEIGHT, ColorType::Rgb8)?;

    println!("File `lut.png` was created! Edit the colors of this file and load it as a LUT. See the LUT example.");

    Ok(())
}
