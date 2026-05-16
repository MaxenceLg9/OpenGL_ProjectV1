use std::io;
use std::io::Cursor;
use std::ops::{Add, Deref, Div, Mul};
use bitvec::macros::internal::funty::Fundamental;
use image::{DynamicImage, GenericImage, ImageFormat, ImageReader, Rgba, RgbaImage};
use noise::{NoiseFn, Perlin};
use shared::math::{get_continentalness, get_erosion, get_peaks_and_valleys, get_terrain_height};
use shared::print_base;

#[path="../client/mod.rs"] pub mod client;
#[path="../server/mod.rs"] pub mod server;


#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

pub fn main() -> io::Result<()> {
    // #[cfg(feature = "dhat-heap")]
    // let _profiler = dhat::Profiler::new_heap();
    let mut rgba_image = RgbaImage::new(500,500);
    let perlin = Perlin::new(1);
    for x in 0..500 {
        for y in 0..500 {
            let noises = get_erosion(&perlin, x as f64, y as f64) * 255.0;
            let value = noises as u8;
            rgba_image.get_pixel_mut(x,y).0 = [value,value,value, 255];
        }
    }
    rgba_image.save_with_format("image.png",ImageFormat::Png).unwrap();
    
    Ok(())
}