use crate::common::world::block::block::{BlockId, BlockState, BlocksRegistries};
use mlua::Value::Boolean;
use noise::{NoiseFn, Perlin, Worley};
use std::collections::HashMap;
use std::ops::{Add, Div, Mul, Sub};
use std::sync::Arc;

use mlua::{Lua, UserData, UserDataFields, UserDataMethods};

pub struct SpinePoint {
    noise: f64,
    value: f64,
}

pub struct Generator {
    erosion_points: Vec<SpinePoint>,
    continentalness_points: Vec<SpinePoint>,
    peaks_and_valleys_points: Vec<SpinePoint>,
    perlin: Perlin,
    registries: Arc<HashMap<String, BlocksRegistries>>,
    seed: u32,
}

// use noise for biomes and combine biomes with the noises below
// cellular noises for carves
impl Generator {
    pub fn new(seed: u32, registries: Arc<HashMap<String, BlocksRegistries>>) -> Self {
        let erosion_points = vec![SpinePoint { noise: 0.0, value: -0.2 }, SpinePoint { noise: 0.4, value: -0.1 }, SpinePoint { noise: 0.6, value: 0.0 }, SpinePoint { noise: 0.7, value: 0.1 }, SpinePoint { noise: 1.0, value: 0.15 }];
        // change how the spine points work by associating a frequency with a fixed value and linearly interpolating the values
        // instead of doing ax + b equation
        let continentalness_points = vec![
            SpinePoint { noise: 0.0, value: -150.0 },
            SpinePoint { noise: 0.10, value: -100.0 },
            SpinePoint { noise: 0.20, value: -60.0 },
            SpinePoint { noise: 0.30, value: -18.0 },
            SpinePoint { noise: 0.40, value: 0.0 },
            SpinePoint { noise: 0.6, value: 30.0 },
            SpinePoint { noise: 0.65, value: 45.0 },
            SpinePoint { noise: 0.7, value: 80.0 },
            SpinePoint { noise: 0.8, value: 110.0 },
            SpinePoint { noise: 0.95, value: 180.0 },
            SpinePoint { noise: 1.0, value: 250.0 },
        ];

        let peaks_and_valleys_points = vec![SpinePoint { noise: 0.0, value: -0.1 }, SpinePoint { noise: 0.25, value: -0.05 }, SpinePoint { noise: 0.5, value: 0.0 }, SpinePoint { noise: 0.8, value: 0.1 }, SpinePoint { noise: 1.0, value: 0.2 }];
        Self {
            erosion_points,
            continentalness_points,
            peaks_and_valleys_points,
            perlin: Perlin::new(seed),
            seed,
            registries,
        }
    }

    pub fn get_terrain_height(&self, x: i32, y: i32) -> f64 {
        let xf = x as f64;
        let yf = y as f64;

        let c_noise = self.get_c_noise(xf, yf);
        let continentalness = self.get_continentalness(c_noise);

        let e_noise = self.get_e_noise(xf, yf);
        let erosion = self.get_erosion(e_noise, c_noise, continentalness);

        let pv_noise = self.get_pv_noise(xf, yf);
        let peaks_and_valleys = self.get_peaks_and_valleys(pv_noise, c_noise, continentalness);

        let base_height = continentalness + erosion + peaks_and_valleys;
        base_height
    }

    pub fn get_c_noise(&self, x: f64, y: f64) -> f64 {
        self.get_perlin_2d(x, y, 0.001, 0.4)
    }

    pub fn get_e_noise(&self, x: f64, y: f64) -> f64 {
        self.get_perlin_2d(x, y, 0.004, 0.2)
    }

    pub fn get_pv_noise(&self, x: f64, y: f64) -> f64 {
        self.get_perlin_2d(x, y, 0.016, 0.12)
    }

    pub fn get_perlin_2d(&self, x: f64, y: f64, f: f64, step: f64) -> f64 {
        self.perlin.get([x.add(step).mul(f), y.add(step).mul(f)]) * 0.5 + 0.5
    }

    pub fn get_perlin_3d(&self, x: f64, y: f64, z: f64) -> f64 {
        self.perlin.get([x, y, z])
    }

    pub fn get_perlin_height(&self, x: f64, z: f64) -> f64 {
        self.get_perlin_2d(x, z, 0.003, 0.0) * 120.0 + 300.0
    }

    pub fn peaks_and_valleys(&self, x: f64, y: f64) -> f64 {
        let mut f = 0.01;
        let mut ret = 0.0_f64;
        for i in 0..4 {
            ret += alpha(ret, i) * self.perlin.get([x.mul(f), y.mul(f)]);
            f *= 2.0;
        }
        ret
    }

    pub fn get_3d(&self, x: f64, y: f64, z: f64) -> Option<BlockId> {
        let mut noise = 0.0;
        let mut offset = 0.4;
        let mut frequency = 0.05;
        for _ in 0..3 {
            noise += self
                .perlin
                .get([x.add(offset * noise).mul(frequency), y.add(offset * noise).mul(frequency), z.add(offset * noise).mul(frequency)])
                * frequency;
            frequency /= 10.0;
            offset *= 2.0;
        }

        if noise > 0.0 { self.registries.get("blocks")?.get_by_name("minecraft:air") } else { self.registries.get("blocks")?.get_by_name("minecraft:grass") }
    }

    pub fn density_of(&self, x: f64, y: f64, z: f64, height: f64, weight_bias: f64, height_bias: f64) -> Option<BlockId> {
        if y > height + 100.0 {
            return self.registries.get("blocks")?.get_by_name("minecraft:air");
        }
        if y < height - 100.0 {
            return self.registries.get("blocks")?.get_by_name("minecraft:grass");
        }
        let mut noise = 0.0;
        let mut offset = 0.02;
        let mut frequency = 0.03;
        let mut power = 4.0.div(7.0);
        for _ in 0..3 {
            noise += self
                .perlin
                .get([x.add(offset).mul(frequency), y.add(offset).mul(frequency), z.add(offset).mul(frequency)])
                * power;
            frequency *= 0.2;
            power /= 2.0;
            offset *= 4.0;
        }

        noise = self.interpolate_noise(noise, y, height, weight_bias, height_bias);

        if noise > 0.0 {
            if y < 0.0 {
                self.registries.get("blocks")?.get_by_name("minecraft:platinum_ore")
            } else if y < height {
                self.registries.get("blocks")?.get_by_name("minecraft:air")
            } else {
                self.registries.get("blocks")?.get_by_name("minecraft:air")
            }
        } else {
            if y >= height {
                self.registries.get("blocks")?.get_by_name("minecraft:stone")
            } else {
                self.registries.get("blocks")?.get_by_name("minecraft:grass")
            }
        }
    }

    /// if y < height - 100, should be -1
    /// if y > height - 100 and y < height, should be between close to 0
    /// if y > height + 5, should be > 1
    pub fn interpolate_noise(&self, noise: f64, y: f64, height: f64, weight_bias: f64, height_bias: f64) -> f64 {
        let weight = self.weight_noise(y, height) * weight_bias;
        let weighted_noise = noise * weight;
        let weighted_height = y.sub(height) / height.abs().max(50.0) * (1.0 - weight) * height_bias;
        weighted_height + weighted_noise
    }

    // pub fn weight_noise(&self, y : f64, height : f64) -> f64 {
    //     let x = y.sub(height * 0.9)  / height.mul(0.9);
    //     -20.0 * x.powi(2) / std::f64::consts::E.powf(2.0 * x.abs() + 1.0) + 1.0
    // }

    pub fn weight_noise(&self, y: f64, height: f64) -> f64 {
        let x = y.sub(height) / height.abs().max(30.0);
        let exp = 0.7;
        (-5.5 * x.abs().powf(exp) / std::f64::consts::E.powf(exp * x.abs() + 1.0) + 1.0).max(0.0)
    }

    pub fn get_continentalness(&self, c_noise: f64) -> f64 {
        spine_noise_into_points(c_noise, &self.continentalness_points)
    }

    pub fn get_peaks_and_valleys(&self, pv_noise: f64, c_noise: f64, continentalness: f64) -> f64 {
        continentalness.max(10.0) * spine_noise_into_points((pv_noise * (c_noise * 0.5 + 0.5)) + 0.5 * (1.0 - c_noise), &self.peaks_and_valleys_points)
    }
    pub fn get_erosion(&self, e_noise: f64, c_noise: f64, continentalness: f64) -> f64 {
        continentalness.max(40.0) * spine_noise_into_points((e_noise * (c_noise * 0.5 + 0.5)) + 0.5 * (1.0 - c_noise), &self.erosion_points)
    }

    pub fn get_worley_2d(&self, x: f64, y: f64, f: f64) -> f64 {
        Worley::new(self.seed).get([x.mul(f), y.mul(f)]) * 0.5 + 0.5
    }
}

pub fn spine_noise_into_points(noise: f64, spine_points: &Vec<SpinePoint>) -> f64 {
    for i in 0..spine_points.len() {
        let point = &spine_points[i];

        if noise < point.noise {
            let (last_noise, last_value) = if let Some(point) = spine_points.get(i - 1) { (point.noise, point.value) } else { (0.0, 0.0) };
            return (point.value - last_value) * default_function((noise - last_noise) / (point.noise - last_noise)) + last_value;
        }
    }
    if let Some(point) = spine_points.last() {
        let (last_noise, last_value) = if let Some(point) = spine_points.get(spine_points.len() - 2) { (point.noise, point.value) } else { (0.0, 0.0) };
        return (point.value - last_value) * default_function((noise - last_noise) / (point.noise - last_noise)) + last_value;
    }
    0.0
}

// f(x)=((2 x^(2))/(1+(2 x-1)^(2)))
pub fn default_function(x: f64) -> f64 {
    let numerator = x.powi(2).mul(2.0);
    let denominator = (x.mul(2.0) - 1.0).powi(2) + 1.0;
    numerator / denominator
    // ((2.0 * (x-1.0))/(1.0+(x-1.0).powf(2.0)))+1.0
}

pub fn alpha(t: f64, o: i32) -> f64 {
    if o == 0 {
        return 1.0;
    }
    let mut frequency: f64 = 1.0;
    let h = 0.75;
    for _i in 0..o {
        frequency *= 2.0;
    }
    t * frequency.powf(-h)
}

impl UserData for Generator {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        // &self, not &mut self -> use add_method, not add_method_mut
        methods.add_method("get_terrain_height", |_lua, this, (x, y): (i32, i32)| Ok(this.get_terrain_height(x, y)));
        methods.add_method("interpolate_noise", |_lua, this, (noise, y, height, weight_bias, height_bias): (f64, f64, f64, f64, f64)| Ok(this.interpolate_noise(noise, y, height, weight_bias, height_bias)));
        methods.add_method("get_perlin_3d", |_lua, this, (x, y, z): (f64, f64, f64)| Ok(this.get_perlin_3d(x, y, z)));
    }
}
