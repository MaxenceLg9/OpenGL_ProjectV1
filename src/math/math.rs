use std::ops::{Mul};
use noise::{NoiseFn, Perlin};

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

pub fn get_terrain_height(perlin: &Perlin, x: i32, y: i32) -> f64 {
    let xf = x as f64;
    let yf = y as f64;
    let base_height = peaks_and_valleys(perlin, xf, yf) * 150.0 * get_erosion(perlin, xf, yf) + 150.0;
    base_height
}

pub fn get_erosion(perlin: &Perlin, x : f64, y : f64) -> f64 {
    let f = 0.001;
    let e_noise = perlin.get([x * f, y * f]);
    compute_erosion(e_noise)
}

pub fn compute_erosion(e_noise : f64) -> f64 {
    if e_noise < 0.0 {
        0.2 + 0.2 * default_function(1.0 + e_noise)
    } else if e_noise < 0.5 {
        0.4 + 0.4 * default_function(e_noise * 2.0)  // or divided by 0.5
    }
    else if e_noise < 0.8 {
        0.8 + 0.2 * default_function((e_noise - 0.5) / 0.3)
    }
    else {
        1.0 + default_function((e_noise - 0.8) * 5.0)
    }
}

// f(x)=((2 x^(2))/(1+(2 x-1)^(2)))
pub fn default_function(x : f64) -> f64 {
    let numerator = x.powi(2).mul(2.0);
    let denominator = (x.mul(2.0) - 1.0).powi(2) + 1.0;
    numerator / denominator
    // ((2.0 * (x-1.0))/(1.0+(x-1.0).powf(2.0)))+1.0
}

pub fn peaks_and_valleys(perlin : &Perlin, x : f64, y : f64) -> f64 {
    let mut f = 0.01;
    let mut ret = 0.0_f64;
    for i in 0..4 {
        ret += alpha(ret, i) * perlin.get([x.mul(f), y.mul(f)]);
        f *= 2.0;
    }
    ret
}


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