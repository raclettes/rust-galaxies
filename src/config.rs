use std::{env, fs::File, io::Read, path::Path};

use rand::{Rng, distributions::Uniform, prelude::Distribution};
use serde::{Deserialize, Serialize};
use serde_yaml::Result;

use crate::App;

#[derive(Serialize, Deserialize)]
pub struct MinMax {
    min: f64,
    max: f64,
}

#[derive(Serialize, Deserialize)]
pub struct Star {
    mass: f64,
    color: [f32; 4],
}

#[derive(Serialize, Deserialize)]
pub struct Planets {
    distance: MinMax,
    mass: MinMax,
    number: i32
}

#[derive(Serialize, Deserialize)]
pub struct Galaxy {
    star: Star,
    planets: Planets,
    position: [f64; 2],
    direction: f64,
}

#[derive(Serialize, Deserialize)]
pub struct Configuration {
    galaxies: Vec<Galaxy>,
    scale: f64,
    time_scale: f64
}

pub fn load_config(app: &mut App) -> Result<()> {
    let mut path = env::current_dir().unwrap();
    path = path.join(Path::new("..")).join(Path::new("config.yaml"));

    let mut file = File::open(path).unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    let config: Configuration = serde_yaml::from_str(&data)?;

    for galaxy in config.galaxies {
        app.add_particle(
            galaxy.star.mass,
            0.0,
            0.0,
            galaxy.position[0],
            galaxy.position[0],
            galaxy.star.color,
        );

        let mut rng = rand::thread_rng();
        let distance = Uniform::from(galaxy.planets.distance.min..=galaxy.planets.distance.max);
        let mass = Uniform::from(galaxy.planets.distance.min..=galaxy.planets.distance.max);
        let col = Uniform::from(1..=255);

        for _ in 0..galaxy.planets.number {
            let r = distance.sample(&mut rng) as f64;
            let m = mass.sample(&mut rng) as f64;

            let v = ((galaxy.star.mass + m) / r).sqrt();

            let theta: f64 = rng.gen::<f64>() * 2.0 * std::f64::consts::PI;

            let v_x = theta.sin() * v * galaxy.direction;
            let v_y = theta.cos() * v * galaxy.direction;

            let r_x = theta.cos() * r;
            let r_y = -theta.sin() * r;

            app.add_particle(
                m,
                v_x,
                v_y,
                galaxy.position[0] + r_x,
                galaxy.position[1] + r_y,
                [
                    col.sample(&mut rng) as f32 / 255.0,
                    col.sample(&mut rng) as f32 / 255.0,
                    col.sample(&mut rng) as f32 / 255.0,
                    1.0,
                ],
            );
        }
    }

    app.scale = config.scale;
    app.time_scale = config.time_scale;

    Ok(())
}
