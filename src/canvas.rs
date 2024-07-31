// Using Perlin noise for organic-
/// looking movement on particles.
/// Also, drawing evenly spread
/// sticks on the back which depict
/// the current flow of the particles.
/// Although the positions for the sticks
/// are fixed, angles are taken from
/// the closest particles.
use lerp::Lerp;
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
use web_sys::{
    console, CanvasRenderingContext2d,
    HtmlCanvasElement,
};

// NOTE: Tried using 'KdTree' to create
// a lookup table for particle positions
// hoping to improve performance, but it
// became ratther slower...
//
// use kdtree::distance::squared_euclidean;
// use kdtree::KdTree;

use crate::utils::{
    color_change_intensity_hex, debounce,
    device_pixel_ratio, get_canvas_size, get_ctx,
    get_window, lazy_round,
};

const NUM_OF_PARTICLES: usize = 150;
const SECOND_COLOR_INTENSITY: f64 = 0.5;

const SPEED: f64 = 3.0;

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

// As a browser resizes, we get
// new width and height.
// When it happens, we want
// to calculate a new 'unit_size'
// with which we determine
// how many grids to draw
// on the screen. This number
// depends whether it is
// for desktop or mobile.
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
    pub particle_size: f64,
    pub num_of_horizontal_grids: usize,
    pub num_of_vertical_grids: usize,
}

impl Canvas {
    pub fn new(
        el: HtmlCanvasElement,
        bgcolor: String,
        color: String,
    ) -> Self {
        let ctx = get_ctx(&el).unwrap();
        let dpr: f64 = device_pixel_ratio();
        let color2 = color_change_intensity_hex(
            &color,
            SECOND_COLOR_INTENSITY,
        );

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
            particle_size: 0.1,
            num_of_horizontal_grids: 10,
            num_of_vertical_grids: 10,
        }
    }

    // Although we want 'update_size' to run
    // as browser size changes, we want
    // to debounce the event by 500 msec.
    pub fn register_listeners(&mut self) {
        let self_rc =
            Rc::new(RefCell::new(self.clone()));

        let mut debounced_update_size = debounce(
            move || {
                let mut canvas = self_rc.borrow_mut();
                canvas.update_size();
            },
            Duration::from_millis(500),
        );

        let callback =
            Closure::wrap(Box::new(move || {
                debounced_update_size();
            })
                as Box<dyn FnMut()>);

        get_window()
            .expect("No window")
            .set_onresize(Some(
                callback.as_ref().unchecked_ref(),
            ));

        callback.forget(); // prevent closure being dropped soon
    }

    // Called when browser size changes.
    pub fn update_size(&mut self) {
        let (w, h): (f64, f64) =
            get_canvas_size(&self.el);

        self.frame = 0;

        let (particle_size, grid_size) = if w < 768.0
        {
            (PARTICLE_SIZE_MOBILE, GRID_SIZE_MOBILE)
        } else {
            (PARTICLE_SIZE_DESKTOP, GRID_SIZE_DESKTOP)
        };

        let width: f64 = w * self.dpr;
        let height: f64 = h * self.dpr;

        let unit_size = width / grid_size;

        self.unit_size = unit_size;
        self.particle_size = particle_size;

        self.num_of_horizontal_grids =
            (height / unit_size).ceil() as usize;
        self.num_of_vertical_grids =
            (width / unit_size).ceil() as usize;

        self.particles = generate_particles(
            width,
            height,
            NUM_OF_PARTICLES,
        );

        console::log_1(
            &("[canvas] Updating canvas size".into()),
        );

        console::log_1(
            &(format!(
                "[canvas] {} x {}",
                lazy_round(width),
                lazy_round(height)
            )
            .into()),
        );

        console::log_1(
            &(format!(
                "[canvas] particle_size: {}",
                lazy_round(particle_size)
            )
            .into()),
        );

        console::log_1(
            &(format!(
                "[canvas] grid_size: {}",
                lazy_round(grid_size)
            )
            .into()),
        );

        // mosaikekkan
        // if let Err(err) = get_window().map(|window| {
        //     window.alert_with_message(
        //         &(format!(
        //             "width: {}",
        //             lazy_round(width)
        //         )),
        //     )
        // }) {
        //     console::log_1(
        //         &(format!("ERR: {}", err).into()),
        //     )
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

        for p in &mut self.particles {
            let w = self.width;
            let h = self.height;

            // Keep using random values when
            // generating noise, otherwise,
            // all particles would have the same
            // positions and angles which
            // would not look dynamic at all.
            let noise_val = self.noise.get([
                (p.x / w) + rng.gen_range(-0.1, 0.1),
                (p.y / h) + rng.gen_range(-0.1, 0.1),
                self.frame as f64 / 100.0,
            ]);

            let angle = noise_val * PI * 2.0;

            let (dx, dy) = (
                SPEED * angle.cos(),
                SPEED * angle.sin(),
            );

            let size = self.particle_size;

            p.x += dx * size;
            p.y += dy * size;

            if p.x < -size {
                p.x = w + size;
            }
            if p.y < -size {
                p.y = h + size;
            }
            if p.x > w + size {
                p.x = -size;
            }
            if p.y > h + size {
                p.y = -size;
            }
        }
    }

    // Repeatedly called from 'Proxy.run'.
    pub fn draw(&mut self) {
        self.ctx.set_fill_style(
            &self.bgcolor.as_str().into(),
        );
        self.ctx.fill_rect(
            0_f64,
            0_f64,
            self.width,
            self.height,
        );

        // ------------------------------------
        // Sticks
        // ------------------------------------
        // Equally spreads 'sticks' are to be
        // drawn on the screen to serve as
        // an indicator for where particles
        // are heading toward.
        // They have fixed positions, but for
        // angles, it looks for the closest
        // particle, and use that angle.
        // For smoother animations, we are
        // taking 2 particles to interporate
        // the average for these 2 particles.
        self.ctx.set_stroke_style(
            &self.color2.as_str().into(),
        );
        self.ctx.set_line_width(1.0);

        let ripple_effect_range_max =
            8.0 * self.unit_size;

        // Tried using 'KdTree' hoping to improve
        // performance, but it became rather
        // slower...
        //
        // mosaikekkan
        // let mut tree = KdTree::new(2);
        // for (index, particle) in
        //     self.particles.iter().enumerate()
        // {
        //     tree.add([particle.x, particle.y], index)
        //         .unwrap();
        // }

        for i in 0..self.num_of_horizontal_grids {
            let y = i as f64 * self.unit_size;
            for j in 0..self.num_of_vertical_grids {
                let x = j as f64 * self.unit_size;

                // Find the two closest particles to the stick.
                let mut closest_part = [
                    Rc::new(RefCell::new(
                        &self.particles[0],
                    )),
                    Rc::new(RefCell::new(
                        &self.particles[1],
                    )),
                ];
                let mut closest_dist =
                    [f64::MAX, f64::MAX];

                // mosaikekkan
                // let indices = tree
                //     .within(
                //         &[x, y],
                //         ripple_effect_range_max
                //             * ripple_effect_range_max,
                //         &squared_euclidean,
                //     )
                //     .unwrap();

                // for (_, &index) in indices {
                for p in &self.particles {
                    // let p = &self.particles[index];
                    let dist = ((p.x - x).powi(2)
                        + (p.y - y).powi(2))
                    .sqrt();

                    if dist < closest_dist[0] {
                        closest_dist[1] =
                            closest_dist[0];
                        closest_part[1] =
                            closest_part[0].clone();
                        closest_dist[0] = dist;
                        closest_part[0] =
                            Rc::new(RefCell::new(p));
                    } else if dist < closest_dist[1] {
                        closest_dist[1] = dist;
                        closest_part[1] =
                            Rc::new(RefCell::new(p));
                    }
                }

                // If we were to just use the angle
                // of the closest particle, the animation
                // will not look smooth, and it will have
                // jagged appearance. It is because they are
                // updated only once per stick per frame,
                // based on the closest particle at that
                // moment in time. This can cause adrupt
                // changes in angle from frame to frame,
                // and will lead to jagged appearance.
                //
                // To prevent this, we want to interpolate
                // the angle based on the distance to the
                // to closest particles. We are using
                // a weighted average of the angles of
                // the particles where the weights are
                // based on the distance of each particle
                // to the stick. This would result
                // in a more gradual change in angle
                // for the stick.
                let mut angle = 0.0;
                let total_dist =
                    closest_dist[0] + closest_dist[1];

                if total_dist > 0.0 {
                    let weight_0 =
                        closest_dist[1] / total_dist;
                    let weight_1 = 1.0 - weight_0;
                    let part_0 =
                        closest_part[0].borrow();
                    let part_1 =
                        closest_part[1].borrow();
                    angle = part_0.angle * weight_0
                        + part_1.angle * weight_1;
                }

                // If the closest distance to particles
                // is more than 8 units away, we want
                // the length of the stick to be fixed
                // to 2px. If not, then have
                // a proportional size; closer to
                // the particles, bigger it gets.
                let dist_ratio = total_dist
                    / ripple_effect_range_max;

                let stick_size = self
                    .unit_size
                    .lerp(2.0, dist_ratio)
                    .max(2.0)
                    .min(self.unit_size);

                self.ctx.save();
                self.ctx
                    .translate(x, y)
                    .unwrap_or(());
                self.ctx.rotate(angle).unwrap_or(());
                self.ctx.begin_path();
                self.ctx.move_to(0_f64, 0_f64);
                self.ctx.line_to(stick_size, 0_f64);
                self.ctx.stroke();
                self.ctx.restore();
            }
        }

        // ------------------------------------
        // Particles
        // ------------------------------------
        self.ctx.set_fill_style(
            &self.color.as_str().into(),
        );

        let radius = self.particle_size / 2.0;

        for p in &self.particles {
            // Translate the canvas to the particle position.
            self.ctx.save();
            self.ctx
                .translate(p.x, p.y)
                .unwrap_or(());

            // Rotate the canvas based on the particle angle.
            self.ctx.rotate(p.angle).unwrap_or(());

            self.ctx.begin_path();
            self.ctx
                .arc(
                    0_f64,
                    0_f64,
                    radius,
                    0_f64,
                    2.0 * PI,
                )
                .unwrap_or(());
            self.ctx.fill();

            self.ctx.restore();
        }
    }
}

fn generate_particles(
    width: f64,
    height: f64,
    count: usize,
) -> Vec<Particle> {
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
