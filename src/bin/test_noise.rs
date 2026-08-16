use std::collections::{HashMap, HashSet};
use std::io;
use std::io::Cursor;
use std::ops::{Add, Deref, Div, Mul};
use std::sync::Arc;
use bitvec::macros::internal::funty::Fundamental;
use glam::{IVec2, Vec2};
use image::{DynamicImage, GenericImage, ImageFormat, ImageReader, Rgba, RgbaImage};
use noise::{NoiseFn, Perlin};
use shared::worldgen::Generator;
use shared::print_base;

#[path="../client/mod.rs"] pub mod client;
#[path="../server/mod.rs"] pub mod server;


#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

pub fn main() -> io::Result<()> {
    // #[cfg(feature = "dhat-heap")]
    // let _profiler = dhat::Profiler::new_heap();
    let mut rgba_image = RgbaImage::new(5000,5000);
    let generator = Generator::new(1);
    let interpolated = generator.weight_noise(30.0,500.0);
    print_base!("Interpolated {}", interpolated);
    let mut hashmap = HashMap::new();
    let mut max_value = 0.0;
    for x in 0..1000 {
        for y in 0..1000 {
            let noise = generator.get_terrain_height(x, y);
            // testing
            // let noise = generator.get_continentalness(noise);
            // let h = generator.get_continentalness(noise);
            // let noise = generator.get_perlin_2d(x as f64, y as f64, 0.001, 0.0);
            if noise as f64 > max_value {
                max_value = noise as f64;
            }
            hashmap.insert(IVec2::new(x,y), noise as f64);
        }
    }
    for x in 0..1000 {
        for y in 0..1000 {
            let value = (hashmap.get(&IVec2::new(x, y)).unwrap() / max_value * 255.0) as u8;
            rgba_image.get_pixel_mut(x as u32,y as u32).0 = [value,value,value, 255];
        }
    }
    let datetime = chrono::offset::Local::now();
    rgba_image.save_with_format(format!("image_{}.png", datetime),ImageFormat::Png).unwrap();

    Ok(())
}