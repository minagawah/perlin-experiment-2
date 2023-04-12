/// Drawing particles, positions and angles for which
/// are constantly calculated by Perlin noise.
/// Also, we are drawing sticks to indicate
/// the current flow (angles) of the particles.
/// To do so, we will have grids on the canvas screen
/// where sticks are equally spread. Although sticks
/// have fixed positions, angles are of particles
/// (for sticks to indicate the particle flow).
use noise::{NoiseFn, Perlin};
use rand::distributions::Uniform;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::f64::consts::PI;
use std::rc::Rc;
use std::time::Duration;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{console, CanvasRenderingContext2d, HtmlCanvasElement};

use crate::utils::{
    color_change_intensity_hex, debounce, device_pixel_ratio, get_canvas_size,
    get_ctx, get_window, lazy_round,
};

const NUM_OF_PARTICLES: usize = 150;
const SECOND_COLOR_INTENSITY: f64 = 0.4;
const STICK_SIZE_RATIO: f64 = 0.8;

const PARTICLE_SIZE_MOBILE: f64 = 6.5;
const PARTICLE_SIZE_DESKTOP: f64 = 3.5;

// We want a different number of grids
// for desktop and mobile. For desktop
// has more to show, and we will have
// more grids. For mobile, less grids.
const GRID_SIZE_MOBILE: f64 = 15.0;
const GRID_SIZE_DESKTOP: f64 = 50.0;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Particle {
    x: f64,
    y: f64,
    angle: f64,
}

// When a browser size changes, we want
// to find out its width and height, and
// will have to calculate some values required
// for us to draw particles and sticks.
// For 'unit_size' is calculated based
// on the given width, and is the size
// for each grid. 'stick_size' varies,
// but for now, we want 0.8 of 'unit_size'.
// 'particle_size' can have two variations;
// smaller for desktop, and bigger for mobile.
#[derive(Debug, Clone)]
pub struct Canvas {
    pub dpr: f64,
    pub el: HtmlCanvasElement,
    pub ctx: CanvasRenderingContext2d,
    pub width: f64,
    pub height: f64,
    pub bgcolor: String,
    pub color: String,
    pub color2: String,
    pub noise: Perlin,
    pub frame: i32,
    pub particles: Vec<Particle>,
    pub unit_size: f64,
    pub stick_size: f64,
    pub particle_size: f64,
    pub num_of_horizontal_grids: usize,
    pub num_of_vertical_grids: usize,
}

impl Canvas {
    pub fn new(el: HtmlCanvasElement, bgcolor: String, color: String) -> Self {
        let ctx = get_ctx(&el).unwrap();
        let dpr: f64 = device_pixel_ratio();
        let color2 = color_change_intensity_hex(&color, SECOND_COLOR_INTENSITY);

        ctx.scale(dpr, dpr).unwrap_or(());

        Canvas {
            dpr,
            el,
            ctx,
            width: 100.0,
            height: 100.0,
            bgcolor,
            color,
            color2,
            noise: Perlin::new(),
            frame: 0,
            particles: Vec::new(),
            unit_size: 1.0,
            stick_size: 1.0,
            particle_size: 0.1,
            num_of_horizontal_grids: 10,
            num_of_vertical_grids: 10,
        }
    }

    // We want to run 'update_size' as browser size changes.
    // However, we want to debounce the event by 500 msec.
    pub fn register_listeners(&mut self) {
        let self_rc = Rc::new(RefCell::new(self.clone()));

        let mut debounced_update_size = debounce(
            move || {
                let mut canvas = self_rc.borrow_mut();
                canvas.update_size();
            },
            Duration::from_millis(500),
        );

        let callback = Closure::wrap(Box::new(move || {
            debounced_update_size();
        }) as Box<dyn FnMut()>);

        get_window()
            .expect("No window")
            .set_onresize(Some(callback.as_ref().unchecked_ref()));

        callback.forget(); // prevent the closure from being dropped immediately
    }

    // Called when browser size changes.
    pub fn update_size(&mut self) {
        let (w, h): (f64, f64) = get_canvas_size(&self.el);

        self.frame = 0;

        let (particle_size, grid_size) = if w < 768.0 {
            (PARTICLE_SIZE_MOBILE, GRID_SIZE_MOBILE)
        } else {
            (PARTICLE_SIZE_DESKTOP, GRID_SIZE_DESKTOP)
        };

        let width: f64 = w * self.dpr;
        let height: f64 = h * self.dpr;

        let unit_size = width / grid_size;

        self.unit_size = unit_size;
        self.stick_size = unit_size * STICK_SIZE_RATIO;
        self.particle_size = particle_size;

        self.num_of_horizontal_grids = (height / unit_size).ceil() as usize;
        self.num_of_vertical_grids = (width / unit_size).ceil() as usize;

        self.particles = generate_particles(width, height, NUM_OF_PARTICLES);

        console::log_1(&("[canvas] Updating canvas size".into()));
        console::log_1(
            &(format!(
                "[canvas] {} x {}",
                lazy_round(width),
                lazy_round(height)
            )
            .into()),
        );

        console::log_1(
            &(format!("[canvas] particle_size: {}", lazy_round(particle_size))
                .into()),
        );

        console::log_1(
            &(format!("[canvas] grid_size: {}", lazy_round(grid_size)).into()),
        );

        // if let Err(err) = get_window().map(|window| {
        //     window
        //         .alert_with_message(&(format!("width: {}", lazy_round(width))))
        // }) {
        //     console::log_1(&(format!("ERR: {}", err).into()))
        // }

        self.el.set_width(width as u32);
        self.el.set_height(height as u32);

        self.width = lazy_round(width);
        self.height = lazy_round(height);
    }

    // Repeatedly called from 'Proxy.run'.
    pub fn update(&mut self) {
        self.frame += 1;
        let mut rng = rand::thread_rng();

        for particle in &mut self.particles {
            // Notice it keeps using random values
            // for generating the noise.
            // Otherwise, all the particles
            // would eventually have the same
            // positions and angles, and it would
            // not look dynamic at all.
            let noise_val = self.noise.get([
                (particle.x / self.width) + rng.gen_range(-0.1, 0.1),
                (particle.y / self.height) + rng.gen_range(-0.1, 0.1),
                self.frame as f64 / 100.0,
            ]);

            let angle = noise_val * PI * 2.0;
            let speed = 3.0;
            let (dx, dy) = (speed * angle.cos(), speed * angle.sin());

            particle.x += dx * self.particle_size;
            particle.y += dy * self.particle_size;

            if particle.x < -self.particle_size {
                particle.x = self.width + self.particle_size;
            }
            if particle.y < -self.particle_size {
                particle.y = self.height + self.particle_size;
            }
            if particle.x > self.width + self.particle_size {
                particle.x = -self.particle_size;
            }
            if particle.y > self.height + self.particle_size {
                particle.y = -self.particle_size;
            }
        }
    }

    // Repeatedly called from 'Proxy.run'.
    pub fn draw(&mut self) {
        self.ctx.set_fill_style(&self.bgcolor.as_str().into());
        self.ctx.fill_rect(0_f64, 0_f64, self.width, self.height);

        // ------------------------------------
        // Sticks
        // ------------------------------------
        // We are drawing sticks to indicate the current
        // flow of the particles. While we have a fixed
        // positions for the sticks, we will look for
        // the nearest particle(s) so that we can use
        // the angle for the particle (actually, we are
        // taking 2 particles to interpolate
        // for the average angles for 2 particles).
        self.ctx.set_stroke_style(&self.color2.as_str().into());
        self.ctx.set_line_width(1.0);

        for i in 0..self.num_of_horizontal_grids {
            let y = i as f64 * self.unit_size;
            for j in 0..self.num_of_vertical_grids {
                let x = j as f64 * self.unit_size;

                // Find the two closest particles to the stick.
                let mut closest_part = [
                    Rc::new(RefCell::new(&self.particles[0])),
                    Rc::new(RefCell::new(&self.particles[1])),
                ];

                let mut closest_dist = [f64::MAX, f64::MAX];

                for particle in &self.particles {
                    let distance = ((particle.x - x).powi(2)
                        + (particle.y - y).powi(2))
                    .sqrt();
                    if distance < closest_dist[0] {
                        closest_dist[1] = closest_dist[0];
                        closest_part[1] = closest_part[0].clone();
                        closest_dist[0] = distance;
                        closest_part[0] = Rc::new(RefCell::new(particle));
                    } else if distance < closest_dist[1] {
                        closest_dist[1] = distance;
                        closest_part[1] = Rc::new(RefCell::new(particle));
                    }
                }

                // If we were to just use the angle of the nearest
                // particle, the animation will not look smooth,
                // and it will have jagged appearance.
                // It is because they are updated only once
                // per stick per frame, based on the nearest
                // particle at that moment in time. This can
                // cause adrupt changes in angle from frame
                // to frame, and will lead to jagged appearance.
                //
                // To prevent this, we want to interpolate
                // the angle based on the distance to the
                // to closest particles. We are using
                // a weighted average of the angles of
                // the particles where the weights are based
                // on the distance of each particle to the stick.
                // This would result in a more gradual change
                // in angle for the stick.
                let mut angle = 0.0;
                let total_distance = closest_dist[0] + closest_dist[1];

                if total_distance > 0.0 {
                    let weight0 = closest_dist[1] / total_distance;
                    let weight1 = 1.0 - weight0;
                    let particle0 = closest_part[0].borrow();
                    let particle1 = closest_part[1].borrow();
                    angle =
                        particle0.angle * weight0 + particle1.angle * weight1;
                }

                self.ctx.save();
                self.ctx.translate(x, y).unwrap_or(());
                self.ctx.rotate(angle).unwrap_or(());
                self.ctx.begin_path();
                self.ctx.move_to(0_f64, 0_f64);
                self.ctx.line_to(self.stick_size, 0_f64);
                self.ctx.stroke();
                self.ctx.restore();
            }
        }

        // ------------------------------------
        // Particles
        // ------------------------------------
        self.ctx.set_fill_style(&self.color.as_str().into());

        let radius = self.particle_size / 2.0;

        for particle in &self.particles {
            // Translate the canvas to the particle position.
            self.ctx.save();
            self.ctx.translate(particle.x, particle.y).unwrap_or(());

            // Rotate the canvas based on the particle angle.
            self.ctx.rotate(particle.angle).unwrap_or(());

            self.ctx.begin_path();
            self.ctx
                .arc(0_f64, 0_f64, radius, 0_f64, 2.0 * PI)
                .unwrap_or(());
            self.ctx.fill();

            self.ctx.restore();
        }
    }
}

fn generate_particles(width: f64, height: f64, count: usize) -> Vec<Particle> {
    let mut rng = rand::thread_rng();
    let mut particles = Vec::new();

    let x_range = Uniform::new(0.0, width);
    let y_range = Uniform::new(0.0, height);
    let angle_range = Uniform::new(0.0, 2.0 * PI);

    for _ in 0..count {
        let x = rng.sample(x_range);
        let y = rng.sample(y_range);
        let angle = rng.sample(angle_range);
        particles.push(Particle { x, y, angle });
    }

    particles
}
