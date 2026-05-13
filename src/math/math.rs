// struct IVec3Compare {
//     bool operator()(const glm::ivec3& a, const glm::ivec3& b) const {
//     if (a.x != b.x) return a.x < b.x;
//     if (a.y != b.y) return a.y < b.y;
//     return a.z < b.z;
//     }
// };

use std::ops::{Add, Div, Mul, Sub};
use std::ptr::hash;
use bitvec::macros::internal::funty::Fundamental;
use md4::{Digest, Md4};
use noise::{NoiseFn, Perlin, Seedable};
use noise::core::perlin::perlin_1d;
use crate::print_base;

pub fn alpha(t : f64, o : i32) -> f64 {
    if o == 0 {
        return 1.0;
    }
    let mut frequency : f64 = 1.0;
    let h = 0.75;
    for  _i in 0..o {
        frequency *= 2.0;
    }
    t * frequency.powf(-h)
}

pub fn valley(x : f32) -> f32 {
    1_f32 / 0.2_f32 * 4_f32.powf(((x / 15_f32) * 2_f32 + 0.1_f32).sin())
}

pub fn mountain(x : f32) -> f64 {
    0.1_f64 * 4_f64.powf(noise::Perlin::new(1).get([x as f64 * 0.01_f64, x as f64 * 0.01_f64, 0.0_f64]).sin() * 2_f64 + 5_f64)
}

pub fn terrain2(x : f32, y : f32) -> f64 {
    valley(x) as f64 * noise::Perlin::new(1).get([x as f64 * 0.01_f64, y as f64 * 0.01_f64])
}

// pub fn get_continentalness(perlin: &Perlin, x : f64, y : f64) -> f64 {
//     let mut ret = 0.0_f64;
//     let mut frequency = 0.001;
//     for i in 0..4 {
//         ret += alpha(ret, i) * perlin.get([x * frequency, y * frequency]);
//         frequency *= 2.0;
//     }
//     ret * 10.0
// }
pub fn get_terrain_height(perlin: &Perlin, x: i32, y: i32) -> f64 {
    let xf = x as f64;
    let yf = y as f64;
    let base_height = noised_terrain_default(perlin,xf,yf, 0.005) * 150.0 * get_erosion(perlin,xf,yf) + 150.0;
    base_height
}

pub fn get_erosion(perlin: &Perlin, x : f64, y : f64) -> f64 {
    let e_noise = perlin.get([x * 0.001, y * 0.001]);
    if e_noise < 0.0 {
        0.2 + 0.2 * default_function(e_noise.abs())
    } else if e_noise < 0.5 {
        0.4 + 0.4 * default_function(e_noise * 2.0)  // or divided by 0.5
    }
    else if e_noise < 0.8 {
        0.8 + 0.2 * default_function((e_noise - 0.5) / 0.3)
    }
    else {
        1.0 + default_function(e_noise - 0.8)
    }
}

// f(x)=((2 x^(2))/(1+(2 x-1)^(2)))
fn default_function(x : f64) -> f64 {
    let numerator = x.powi(2).mul(2.0);
    let denominator = (x.mul(2.0) - 1.0).powi(2) + 1.0;
    numerator / denominator
    // ((2.0 * (x-1.0))/(1.0+(x-1.0).powf(2.0)))+1.0
}

pub fn noised_terrain_default(perlin : &Perlin, x : f64, y : f64, mut frequency: f64) -> f64 {
    let mut ret = 0.0_f64;
    for i in 0..4 {
        ret += alpha(ret, i) * perlin.get([x.mul(frequency), y.mul(frequency), 0.0]);
        frequency *= 2.0;
    }
    ret
}

struct SplinePoint {
    n: f64, // Noise input (-1.0 to 1.0)
    v: f64, // Output value (e.g., Height or Multiplier)
}

fn sample_spline(points: &[SplinePoint], input: f64) -> f64 {
    // 1. Handle out-of-bounds
    if input <= points[0].n { return points[0].v; }
    if input >= points[points.len() - 1].n { return points[points.len() - 1].v; }

    // 2. Find the segment (Binary search is overkill for < 10 points, but good for more)
    for i in 0..points.len() - 1 {
        let p1 = &points[i];
        let p2 = &points[i + 1];

        if input >= p1.n && input <= p2.n {
            // 3. Linear Interpolation (lerp)
            let t = (input - p1.n) / (p2.n - p1.n);
            return p1.v + t * (p2.v - p1.v);
        }
    }
    points[points.len() - 1].v
}

// pub fn get_terrain_height(perlin: &Perlin, x: i32, y: i32) -> f64 {
//     let xf = x as f64;
//     let yf = y as f64;
//
//     // Raw Noise Values (-1.0 to 1.0)
//     let c_noise = perlin.get([xf * 0.0005, yf * 0.0005]);
//     let e_noise = perlin.get([xf * 0.002, yf * 0.002]);
//     let pv_noise = perlin.get([xf * 0.01, yf * 0.01]);
//
//     // 1. Continentalness Spline: Defines the "Shelf"
//     let continental_points = [
//         SplinePoint { n: -1.0, v: 20.0 },  // Deep Ocean
//         SplinePoint { n: -0.2, v: 60.0 },  // Coastline
//         SplinePoint { n: 0.0,  v: 64.0 },  // Sea Level
//         SplinePoint { n: 0.2,  v: 128.0 }, // Inland Hills
//         SplinePoint { n: 1.0,  v: 180.0 }, // High Plateaus
//     ];
//     let base_height = sample_spline(&continental_points, c_noise);
//
//     // 2. Erosion Spline: Defines how "Mountainous" an area can be
//     let erosion_points = [
//         SplinePoint { n: -1.0, v: 1.0 },   // Low Erosion = Full Mountains
//         SplinePoint { n: 0.2,  v: 0.5 },   // Mid-Erosion = Rolling Hills
//         SplinePoint { n: 0.8,  v: 0.0 },   // High Erosion = Flat Plains
//     ];
//     let mountain_factor = sample_spline(&erosion_points, e_noise);
//
//     // 3. Peaks & Valleys: Calculate detail and apply the mask
//     // We use .abs() to create "Ridged" mountains
//     let detail = (1.0 - pv_noise.abs()) * 40.0;
//
//     // Final result
//     base_height + (detail * mountain_factor)
// }


pub fn get_continentalness(perlin: &Perlin, x : f64, y : f64, frequency : f64) -> f64 {
    let continentalness = perlin.get([x * frequency, y * frequency]);
    if continentalness < -0.8 {
        30.0 + continentalness * 10.0
    } else if continentalness < -0.5 {
        let t = (continentalness - 0.5) / (0.8 - 0.5);
        return 70.0 + t * (70.0 - 30.0);
        70.0 + continentalness.abs().mul(10.0).log10() * 20.0
    } else if continentalness < 0.0 {
        let t = (continentalness) / (0.5);
        return 100.0 + t * (100.0 - 70.0);
        130.0 + continentalness * 100.0
    } else {
        let t = (continentalness) / (1.0);
        return 130.0 + t * (130.0 - 100.0);
    }
}

pub fn get_peaks_and_valleys(perlin: &Perlin, x : f64, y : f64, frequency : f64) -> f64 {
    perlin.get([x * frequency, y * frequency])
}

pub fn terrain(perlin: &Perlin, x : i32, y : i32) -> f64 {
    //        return noised_terrain_default(x,y) * Utils::mountain(x + y) + 200;
    5_f64.powf(get_terrain_height(perlin, x, y) * 5_f64 + 1_f64)
}