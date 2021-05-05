extern crate glutin_window;
extern crate graphics;
extern crate num_bigint;
extern crate opengl_graphics;
extern crate piston;
extern crate serde;
extern crate serde_yaml;
mod config;

use core::panic;
use std::{collections::HashSet};

use glutin_window::GlutinWindow as Window;
use graphics::color::{BLACK, YELLOW};
use opengl_graphics::{GlGraphics, OpenGL};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use piston::{
    event_loop::{EventSettings, Events},
    Button, Key, PressEvent, ReleaseEvent,
};
use config::{load_config};


type ImplIteratorMut<'a, Item> =
    ::std::iter::Chain<::std::slice::IterMut<'a, Item>, ::std::slice::IterMut<'a, Item>>;
trait SplitOneMut {
    type Item;

    fn split_one_mut(
        &'_ mut self,
        i: usize,
    ) -> (&'_ mut Self::Item, ImplIteratorMut<'_, Self::Item>);
}

impl<T> SplitOneMut for [T] {
    type Item = T;

    fn split_one_mut(
        &'_ mut self,
        i: usize,
    ) -> (&'_ mut Self::Item, ImplIteratorMut<'_, Self::Item>) {
        let (prev, current_and_end) = self.split_at_mut(i);
        let (current, end) = current_and_end.split_at_mut(1);
        (&mut current[0], prev.iter_mut().chain(end))
    }
}

pub struct Particle {
    pub mass: f64,
    pub vx: f64,
    pub vy: f64,
    pub x: f64,
    pub y: f64,
    pub color: [f32; 4],
    pub to_destroy: bool,
}

impl Particle {
    fn force_from(&mut self, other: &Particle) -> (f64, f64) {
        if other.mass < 50.0 {
            return (0.0, 0.0);
        }

        let distance = self.distance_to(other);
        if self.radius() + other.radius() > distance {
            return (0.0, 0.0);
        }

        let f = self.mass * other.mass / distance.powi(2);
        let theta = (other.y - self.y).atan2(other.x - self.x);

        let fx = theta.cos() * f;
        let fy = theta.sin() * f;

        (fx, fy)
    }

    fn distance_to(&mut self, other: &Particle) -> f64 {
        ((other.x - self.x).powi(2) + (other.y - self.y).powi(2)).sqrt()
    }

    fn radius(&self) -> f64 {
        self.mass.log10() / 5.0
    }
}

impl PartialEq for Particle {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x
            && self.y == other.y
            && self.vx == other.vx
            && self.vy == other.vy
            && self.mass == other.mass
            && self.color == other.color
    }
}

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    particles: Vec<Particle>,
    scale: f64,

    cx: f64,
    cy: f64,

    time_scale: f64
}

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        let (x, y) = (args.window_size[0] / 2.0, args.window_size[1] / 2.0);
        let particles = &self.particles;
        let scale = &self.scale;
        let cx = &self.cx;
        let cy = &self.cy;
        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BLACK, gl);

            let transform = c.transform.trans(x, y).trans(-cx * scale, -cy * scale); // .trans(-particles[0].x * scale, -particles[0].y * scale);

            // Draw a box rotating around the middle of the screen.
            // rectangle(RED, square, transform, gl);
            for particle in particles {
                let px = particle.x * scale - (cx * scale) + x;
                let py = particle.y * scale - (cy * scale) + y;

                if px < 0.0 || px > args.window_size[0] || py < 0.0 || py > args.window_size[1] {
                    continue;
                }

                let rad = particle.radius() * scale;
                circle_arc(
                    particle.color,
                    rad,
                    0.0,
                    f64::_360(),
                    rectangle::square(-rad/2.0, -rad/2.0, rad),
                    transform.trans(particle.x * scale, particle.y * scale),
                    gl,
                );
            }
        });
    }

    fn update(&mut self, args: &UpdateArgs, keys: &HashSet<Key>) {
        let dt = args.dt * self.time_scale;
        // Update view
        if keys.contains(&Key::Up) {
            self.cy -= 2.0 / self.scale;
        }

        if keys.contains(&Key::Down) {
            self.cy += 2.0 / self.scale;
        }

        if keys.contains(&Key::Left) {
            self.cx -= 2.0 / self.scale;
        }

        if keys.contains(&Key::Right) {
            self.cx += 2.0 / self.scale;
        }

        if keys.contains(&Key::Z) {
            self.scale /= 1.1;
        }

        if keys.contains(&Key::X) {
            self.scale *= 1.1;
        }

        if keys.contains(&Key::S) {
            self.time_scale *= 1.1;
        }

        if keys.contains(&Key::A) {
            self.time_scale /= 1.1;
        }

        for p1idx in 0..self.particles.len() {
            let (p1, particles) = self.particles.split_one_mut(p1idx);

            if p1.to_destroy {
                continue;
            }

            let mut fx: f64 = 0.0;
            let mut fy: f64 = 0.0;

            for p2 in particles {
                if p2.to_destroy {
                    continue;
                }

                let (t_fx, t_fy) = &p1.force_from(p2);

                fx += t_fx;
                fy += t_fy;

                if p1.mass > 50.0 || p2.mass > 50.0 {
                    let rel_velocity =
                        ((p2.vx - p1.vx).powi(2) + (p2.vy - p1.vy).powi(2)).sqrt();
                    let distance = p1.distance_to(p2);
                    if distance < p1.radius() + p2.radius() && rel_velocity > 160.0 {
                        // Combine
                        p2.to_destroy = true;

                        let momentum_x = p1.mass * p1.vx + p2.mass * p2.vx;
                        let momentum_y = p1.mass * p1.vy + p2.mass * p2.vy;

                        if p2.color == YELLOW || p1.color == YELLOW {
                            p1.color = YELLOW;
                        }

                        p1.mass += p2.mass;
                        p1.vx = momentum_x / p1.mass;
                        p1.vy = momentum_y / p1.mass;
                    }
                }
            }

            p1.vx += fx / p1.mass * dt;
            p1.vy += fy / p1.mass * dt;

            p1.x += p1.vx * dt;
            p1.y += p1.vy * dt;
        }

        let mut i = 0;
        while i != self.particles.len() {
            if self.particles[i].to_destroy {
                self.particles.remove(i);
            } else {
                i += 1;
            }
        }
    }

    fn add_particle(&mut self, mass: f64, vx: f64, vy: f64, x: f64, y: f64, color: [f32; 4]) {
        self.particles.push(Particle {
            mass,
            vx,
            vy,
            x,
            y,
            color,
            to_destroy: false,
        });
    }
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new("Galactic", [800, 800])
        .graphics_api(opengl)
        .decorated(true)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl),
        particles: Vec::new(),
        scale: 2.0,
        cx: 0.0,
        cy: 0.0,
        time_scale: 1.0
    };

    match load_config(&mut app) {
        Ok(()) => (),
        Err(err) => {
            panic!("Unable to load configuration, {:?}", err)
        }
    }

    let mut keys: HashSet<Key> = HashSet::new();

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            app.render(&args);
        }

        if let Some(args) = e.update_args() {
            app.update(&args, &keys);
        }

        if let Some(Button::Keyboard(key)) = e.press_args() {
            keys.insert(key);
        }

        if let Some(Button::Keyboard(key)) = e.release_args() {
            keys.remove(&key);
        }
    }
}
