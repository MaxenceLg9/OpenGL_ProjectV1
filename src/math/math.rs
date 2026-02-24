// struct IVec3Compare {
//     bool operator()(const glm::ivec3& a, const glm::ivec3& b) const {
//     if (a.x != b.x) return a.x < b.x;
//     if (a.y != b.y) return a.y < b.y;
//     return a.z < b.z;
//     }
// };

use noise::NoiseFn;

pub fn alpha(t : f64, o : i32) -> f64 {
    if (o == 0) {
        return 1.0;
    }
    let mut frequency : f64 = 1.0;
    let h = 0.75;
    for  i in 0..o {
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
    valley(x) as f64 * noise::Perlin::new(1).get([x as f64 * 0.01_f64, x as f64 * 0.01_f64])
}

pub fn noised_terrain_default(x : i32, y : i32) -> f64 {
    let perlin = noise::Perlin::new(1);
    let mut ret = 0.0_f64;
    let mut frequency = 0.005;
    for i in 0..4 {
        ret += alpha(ret, i) * perlin.get([x as f64 * frequency, y as f64 * frequency, 0.0]);
        frequency *= 2.0;
    }
    ret
}

pub fn terrain(x : i32, y : i32) -> f64 {
    //        return noised_terrain_default(x,y) * Utils::mountain(x + y) + 200;
    5_f64.powf(noised_terrain_default(x, y) * 5_f64 + 1_f64)
}