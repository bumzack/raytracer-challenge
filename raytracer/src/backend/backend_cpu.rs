// extern crate crossbeam;

use std::error::Error;
use std::time::Instant;

use crate::backend::backend::Backend;
use crate::backend::MAX_REFLECTION_RECURSION_DEPTH;
use cpu_kernel_raytracer::camera::{Camera, CameraOps};
use cpu_kernel_raytracer::color::BLACK;
use cpu_kernel_raytracer::ray::RayOps;
use cpu_kernel_raytracer::CpuKernel;
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelIterator;
use raytracer_lib_std::{Canvas, CanvasOps, World, WorldOps};

pub struct BackendCpu {
    multi_core: bool,
    num_cores: usize,
}

impl Backend for BackendCpu {
    fn render_world(&self, world: &mut World, c: &Camera) -> Result<Canvas, Box<dyn Error>> {
        if self.multi_core {
            return self.render_multi_core(world, c);
        } else {
            return self.render_single_core(world, c);
        }
    }
}

impl BackendCpu {
    pub fn new() -> BackendCpu {
        BackendCpu {
            multi_core: false,
            num_cores: 0,
        }
    }

    pub fn enable_multicore(&mut self) {
        self.multi_core = true;
    }
}

impl BackendCpu {
    fn render_single_core(&self, world: &mut World, c: &Camera) -> Result<Canvas, Box<dyn Error>> {
        let start = Instant::now();
        let n_samples = c.get_antialiasing_size();
        let mut jitter_matrix = Vec::new();
        if n_samples == 2 {
            #[rustfmt::skip]
                jitter_matrix = vec![
                -1.0 / 4.0,
                1.0 / 4.0,
                1.0 / 4.0,
                1.0 / 4.0,
                -1.0 / 4.0,
                -1.0 / 4.0,
                1.0 / 4.0,
                -3.0 / 4.0,
            ];
        }
        if n_samples == 3 {
            let two_over_six = 2.0 / 6.0;
            #[rustfmt::skip]
                jitter_matrix = vec![
                -two_over_six,
                two_over_six,
                0.0,
                two_over_six,
                two_over_six,
                two_over_six,
                -two_over_six,
                0.0,
                0.0,
                0.0,
                two_over_six,
                0.0,
                -two_over_six,
                -two_over_six,
                0.0,
                -two_over_six,
                two_over_six,
                -two_over_six,
            ];
        }
        let mut canvas = Canvas::new(c.get_hsize(), c.get_vsize());

        // TODO: remove, when WOrld has lights vector
        let mut lights = Vec::new();
        lights.push(world.get_light().clone());

        if c.get_antialiasing() {
            for y in 0..c.get_vsize() {
                for x in 0..c.get_hsize() {
                    let mut color = BLACK;

                    // Accumulate light for N samples.
                    for sample in 0..n_samples {
                        let delta_x = jitter_matrix[2 * sample] * c.get_pixel_size();
                        let delta_y = jitter_matrix[2 * sample + 1] * c.get_pixel_size();

                        let r = Camera::ray_for_pixel_anti_aliasing(c, x, y, delta_x, delta_y);

                        color = CpuKernel::color_at(world.get_shapes(), &lights, &r, MAX_REFLECTION_RECURSION_DEPTH)
                            + color;
                    }
                    color = color / n_samples as f32;
                    // println!("with AA    color at ({}/{}): {:?}", x, y, color);
                    canvas.write_pixel(x, y, color);
                }
            }
        } else {
            for y in 0..c.get_vsize() {
                for x in 0..c.get_hsize() {
                    let r = Camera::ray_for_pixel(c, x, y);

                    let color = CpuKernel::color_at(world.get_shapes(), &lights, &r, MAX_REFLECTION_RECURSION_DEPTH);
                    // println!("no AA    color at ({}/{}): {:?}", x, y, color);
                    canvas.write_pixel(x, y, color);
                }
            }
        }
        let stopped = Instant::now();
        println!(
            "\n\ncpu single core     duration: {:?} \n\n",
            stopped.duration_since(start)
        );
        Ok(canvas)
    }

    fn render_single_core_debug(&self, world: &mut World, c: &Camera) -> Result<Canvas, Box<dyn Error>> {
        let start = Instant::now();

        let mut canvas = Canvas::new(c.get_hsize(), c.get_vsize());

        // TODO: remove, when WOrld has lights vector
        let mut lights = Vec::new();
        lights.push(world.get_light().clone());

        println!(
            "r    c.get_vsize() = {:?},    c.get_hsize()  = {:?}",
            c.get_vsize(),
            c.get_hsize()
        );

        let x = 300;
        let y = 240;

        let r = Camera::ray_for_pixel(c, x, y);
        let color = CpuKernel::color_at(world.get_shapes(), &lights, &r, MAX_REFLECTION_RECURSION_DEPTH);
        println!(
            "ray at ( {} / {} )   origin = {:?},    direction = {:?}",
            x,
            y,
            r.get_origin(),
            r.get_direction()
        );
        println!("color  at ( {} / {} )   c = {:?},     ", x, y, color);

        canvas.write_pixel(x, y, color);

        let stopped = Instant::now();
        println!(
            "\n\ncpu single core     duration: {:?} \n\n",
            stopped.duration_since(start)
        );
        Ok(canvas)
    }

    // TODO: implement using rayon ?!?!!?
    fn render_multi_core(&self, world: &mut World, c: &Camera) -> Result<Canvas, Box<dyn Error>> {
        let start = Instant::now();
        let n_samples = c.get_antialiasing_size();
        let mut jitter_matrix = Vec::new();
        if n_samples == 2 {
            #[rustfmt::skip]
                jitter_matrix = vec![
                -1.0 / 4.0,
                1.0 / 4.0,
                1.0 / 4.0,
                1.0 / 4.0,
                -1.0 / 4.0,
                -1.0 / 4.0,
                1.0 / 4.0,
                -3.0 / 4.0,
            ];
        }
        if n_samples == 3 {
            let two_over_six = 2.0 / 6.0;
            #[rustfmt::skip]
                jitter_matrix = vec![
                -two_over_six,
                two_over_six,
                0.0,
                two_over_six,
                two_over_six,
                two_over_six,
                -two_over_six,
                0.0,
                0.0,
                0.0,
                two_over_six,
                0.0,
                -two_over_six,
                -two_over_six,
                0.0,
                -two_over_six,
                two_over_six,
                -two_over_six,
            ];
        }
        let mut canvas = Canvas::new(c.get_hsize(), c.get_vsize());

        // TODO: remove, when WOrld has lights vector
        let mut lights = Vec::new();
        lights.push(world.get_light().clone());

        canvas.get_pixels_mut().into_par_iter().for_each(|p| {
            let x = p.x;
            let y = p.y;
            if c.get_antialiasing() {
                let mut color = BLACK;

                // Accumulate light for N samples.
                for sample in 0..n_samples {
                    let delta_x = jitter_matrix[2 * sample] * c.get_pixel_size();
                    let delta_y = jitter_matrix[2 * sample + 1] * c.get_pixel_size();

                    let r = Camera::ray_for_pixel_anti_aliasing(c, x, y, delta_x, delta_y);

                    color =
                        CpuKernel::color_at(world.get_shapes(), &lights, &r, MAX_REFLECTION_RECURSION_DEPTH) + color;
                }
                color = color / n_samples as f32;
                // println!("with AA    color at ({}/{}): {:?}", x, y, color);
                p.color.r = color.r;
                p.color.g = color.g;
                p.color.b = color.b;
            } else {
                let r = Camera::ray_for_pixel(c, x, y);

                let color = CpuKernel::color_at(world.get_shapes(), &lights, &r, MAX_REFLECTION_RECURSION_DEPTH);
                // println!("no AA    color at ({}/{}): {:?}", x, y, color);
                p.color.r = color.r;
                p.color.g = color.g;
                p.color.b = color.b;
            }
        });

        let stopped = Instant::now();
        println!(
            "\n\ncpu multicore ({} cores)    duration  {:?}      \n\n",
            self.num_cores,
            stopped.duration_since(start)
        );
        Ok(canvas)
    }
}
