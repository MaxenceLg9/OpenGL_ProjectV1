use std::ops::{Mul};
use noise::{NoiseFn, Perlin};

pub struct SpinePoint {
    x : f64,
    y: f64
}

pub struct Generator {
    erosion_points : Vec<SpinePoint>,
    continentalness_points : Vec<SpinePoint>,
    peaks_and_valleys_points : Vec<SpinePoint>,
    perlin : Perlin
}

impl Generator {
    pub fn new(perlin : Perlin) -> Self {
        let erosion_points = vec![
            SpinePoint {
                x: 0.0,
                y: 0.9
            },
            SpinePoint {
                x: 0.1,
                y: 0.9
            },
            SpinePoint {
                x : 0.6,
                y: 1.0
            },
            SpinePoint {
                x: 0.7,
                y: 1.0
            },
            SpinePoint {
                x: 1.0,
                y: 1.1
            }
        ];
        // change how the spine points work by associating a frequency with a fixed value and linearly interpolating the values
        // instead of doing ax + b equation
        let continentalness_points = vec![
            SpinePoint {
                x: 0.0,
                y: 50.0
            },
            SpinePoint {
                x : 0.15,
                y: 100.0
            },
            SpinePoint {
                x: 0.25,
                y: 125.0
            },
            SpinePoint {
                x: 0.45,
                y: 125.0
            },
            SpinePoint {
                x: 0.55,
                y: 150.0
            },
            SpinePoint {
                x: 0.6,
                y: 170.0
            },
            SpinePoint {
                x: 0.7,
                y: 200.0
            },
            SpinePoint {
                x: 0.8,
                y: 225.0
            },
            SpinePoint {
                x: 0.95,
                y: 300.0
            },
            SpinePoint {
                x: 1.0,
                y : 300.0
            }
        ];
        let peaks_and_valleys_points : Vec<SpinePoint> = vec! [];
        Self {
            erosion_points,
            continentalness_points,
            peaks_and_valleys_points,
            perlin
        }
    }

    pub fn get_terrain_height(&self, x: i32, y: i32) -> i32 {
        let xf = x as f64;
        let yf = y as f64;
        let continentalness = self.get_continentalness(xf,yf);
        let base_height = continentalness * self.get_erosion(xf,yf).powf((continentalness / 100.0).max(0.0).sqrt());
        base_height as i32
    }

    pub fn peaks_and_valleys(&self, x : f64, y : f64) -> f64 {
        let mut f = 0.01;
        let mut ret = 0.0_f64;
        for i in 0..4 {
            ret += alpha(ret, i) * self.perlin.get([x.mul(f), y.mul(f)]);
            f *= 2.0;
        }
        ret
    }

    pub fn get_continentalness(&self, x : f64, y : f64) -> f64 {
        let f = 0.001;
        let e_noise = self.perlin.get([x * f, y * f]) * 0.5 + 0.5;
        // return compute_erosion(e_noise);

        spine_noise_into_points(e_noise, &self.continentalness_points)
    }


    pub fn get_erosion(&self, x : f64, y : f64) -> f64 {
        let f = 0.005;
        let e_noise = self.perlin.get([x * f, y * f]) * 0.5 + 0.5;
        // return compute_erosion(e_noise);

        spine_noise_into_points(e_noise, &self.erosion_points)
    }
}

pub fn spine_noise_into_points(noise : f64, spine_points : &Vec<SpinePoint>) -> f64 {
    for i in 0..spine_points.len() {
        let point = &spine_points[i];

        if noise < point.x {
            let (last_x, last_y) = if let Some(point) = spine_points.get(i - 1) {
                (point.x, point.y)
            } else {
                (0.0, 0.0)
            };
            return (point.y - last_y) * default_function((noise - last_x) / (point.x - last_x)) + last_y;
        }
    }
    if let Some(point) = spine_points.last() {
        let (last_x, last_y) = if let Some(point) = spine_points.get(spine_points.len() - 2) {
            (point.x, point.y)
        } else {
            (0.0, 0.0)
        };
        return (point.y - last_y) * default_function((noise - last_x) / (point.x - last_x)) + last_y;
    }
    0.0
}

// pub fn spine_noise_into_points(noise : f64, spine_points : &Vec<SpinePoint>) -> f64 {
//     for i in 0..spine_points.len() {
//         let point = &spine_points[i];
//         let (last_x, last_y) = if let Some(point) = spine_points.get(i - 1) {
//             (point.x, point.y)
//         } else {
//             (0.0, 0.0)
//         };
//         if noise <= point.x {
//             return (point.y - last_y) * default_function((noise - last_x) / (point.x - last_x)) + last_y;
//         }
//     }
//     if let Some(point) = spine_points.last() {
//         let (last_x, last_y) = if let Some(point) = spine_points.get(spine_points.len() - 2) {
//             (point.x, point.y)
//         } else {
//             (0.0, 0.0)
//         };
//         return (point.y - last_y) * default_function((noise - last_x) / (point.x - last_x)) + last_y;
//     }
//     0.0
// }

// f(x)=((2 x^(2))/(1+(2 x-1)^(2)))
pub fn default_function(x : f64) -> f64 {
    let numerator = x.powi(2).mul(2.0);
    let denominator = (x.mul(2.0) - 1.0).powi(2) + 1.0;
    numerator / denominator
    // ((2.0 * (x-1.0))/(1.0+(x-1.0).powf(2.0)))+1.0
}

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