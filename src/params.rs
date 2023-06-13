use bevy::prelude::*;

use rand::Rng;

pub const NUM_PARAMS: usize = 7;
pub const CHILDREN_MINMAX: [u8; 2] = [3, 6];
pub const LEVELS_MINMAX: [u8; 2] = [2, 5];
pub const TRANSLATION_FACTOR_MINMAX: [f32; 2] = [0.0, 1.0];
pub const ANGLE_MINMAX: [f32; 2] = [0.0, 0.5 * std::f32::consts::PI];
pub const SCALE_MINMAX: [f32; 2] = [0.4, 0.8];
pub const BASE_RADIUS_MINMAX: [f32; 2] = [0.1, 0.3];
pub const LEAF_RADIUS_MINMAX: [f32; 2] = [0.1, 0.5];
const PARAMS_VELOCITY_MAG: f32 = 0.5; // in [0, 1]
const PARAMS_ACCELERATION_MAG: f32 = 0.05; // in [0, 1]

#[derive(Debug, Resource)]
pub struct Params {
    pub children: u8,                  // should be greater than 1
    pub levels: u8,                    // should be greater than 0
    pub child_translation_factor: f32, // must be in [0, 1]
    pub angle_from_parent_branch: f32, // must be in [0, PI / 2]
    pub child_scale: f32,              // must be in [0.4, 0.8]
    pub base_radius: f32,              // must be in [0.1, 0.3]
    pub leaf_radius: f32,              // must be in [0.1, 0.5]
}

impl Default for Params {
    fn default() -> Self {
        Self {
            children: 3,
            levels: 1,
            child_translation_factor: 1.0,
            angle_from_parent_branch: 0.5 * std::f32::consts::PI,
            child_scale: 0.7,
            base_radius: 0.15,
            leaf_radius: 0.4,
        }
    }
}

/// Used to randomly walk the parameter space
#[derive(Resource)]
pub struct ParamsVector {
    pub values: [f32; NUM_PARAMS], // must all be in [0, 1]
    pub magnitude: Option<f32>,    // must be in (0, 1] if Vector, None if Point
}

impl Default for ParamsVector {
    fn default() -> Self {
        Self {
            values: [0.0; NUM_PARAMS],
            magnitude: Some(PARAMS_VELOCITY_MAG),
        }
    }
}

impl ParamsVector {
    pub fn to_params(&self) -> Params {
        Params {
            children: ((CHILDREN_MINMAX[1] - CHILDREN_MINMAX[0]) as f32
                * (self.values[0] + 1.0)
                * 0.5
                + CHILDREN_MINMAX[0] as f32)
                .round() as u8,
            levels: ((LEVELS_MINMAX[1] - LEVELS_MINMAX[0]) as f32 * (self.values[1] + 1.0) * 0.5
                + LEVELS_MINMAX[0] as f32)
                .round() as u8,
            child_translation_factor: (TRANSLATION_FACTOR_MINMAX[1] - TRANSLATION_FACTOR_MINMAX[0])
                * (self.values[2] + 1.0)
                * 0.5
                + TRANSLATION_FACTOR_MINMAX[0],
            angle_from_parent_branch: (ANGLE_MINMAX[1] - ANGLE_MINMAX[0])
                * (self.values[3] + 1.0)
                * 0.5
                + ANGLE_MINMAX[0],
            child_scale: (SCALE_MINMAX[1] - SCALE_MINMAX[0]) * (self.values[4] + 1.0) * 0.5
                + SCALE_MINMAX[0],
            base_radius: (BASE_RADIUS_MINMAX[1] - BASE_RADIUS_MINMAX[0])
                * (self.values[5] + 1.0)
                * 0.5
                + BASE_RADIUS_MINMAX[0],
            leaf_radius: (LEAF_RADIUS_MINMAX[1] - LEAF_RADIUS_MINMAX[0])
                * (self.values[6] + 1.0)
                * 0.5
                + LEAF_RADIUS_MINMAX[0],
        }
    }

    pub fn from_params(params: &Params) -> Self {
        Self {
            values: [
                2.0 * (params.children - CHILDREN_MINMAX[0]) as f32
                    / (CHILDREN_MINMAX[1] - CHILDREN_MINMAX[0]) as f32
                    - 1.0,
                2.0 * (params.levels - LEVELS_MINMAX[0]) as f32
                    / (LEVELS_MINMAX[1] - LEVELS_MINMAX[0]) as f32
                    - 1.0,
                2.0 * (params.child_translation_factor - TRANSLATION_FACTOR_MINMAX[0])
                    / (TRANSLATION_FACTOR_MINMAX[1] - TRANSLATION_FACTOR_MINMAX[0])
                    - 1.0,
                2.0 * (params.angle_from_parent_branch - ANGLE_MINMAX[0])
                    / (ANGLE_MINMAX[1] - ANGLE_MINMAX[0])
                    - 1.0,
                2.0 * (params.child_scale - SCALE_MINMAX[0]) / (SCALE_MINMAX[1] - SCALE_MINMAX[0])
                    - 1.0,
                2.0 * (params.base_radius - BASE_RADIUS_MINMAX[0])
                    / (BASE_RADIUS_MINMAX[1] - BASE_RADIUS_MINMAX[0])
                    - 1.0,
                2.0 * (params.leaf_radius - LEAF_RADIUS_MINMAX[0])
                    / (LEAF_RADIUS_MINMAX[1] - LEAF_RADIUS_MINMAX[0])
                    - 1.0,
            ],
            magnitude: None,
        }
    }

    pub fn add(&mut self, other: &mut Self) {
        for (x, y) in self.values.iter_mut().zip(other.values.iter_mut()) {
            *x += *y;

            if self.magnitude.is_none() {
                // If the ParamsVector is a Point, clamp it within the limits of [-1, 1]
                if *x < -1.0 || *x > 1.0 {
                    *y = -*y;
                    *x = x.clamp(-1.0, 1.0);
                }
            }
        }
    }

    fn normalize(&mut self) {
        if let Some(mag) = self.magnitude {
            let mut len2: f32 = 0.0;
            for v in self.values {
                len2 += v.powf(2.0);
            }

            assert!(len2 > 0.0);

            let len = len2.sqrt();
            for v in &mut self.values {
                *v /= len;
                *v *= mag;
            }
        }
    }

    pub fn nudge(&mut self) {
        let mut rng = rand::thread_rng();
        let mut acceleration = Self {
            values: [0.0; NUM_PARAMS],
            magnitude: Some(PARAMS_ACCELERATION_MAG),
        };

        for v in &mut acceleration.values {
            *v = rng.gen_range(-1.0..1.0);
        }

        acceleration.normalize();

        //println!("Acc {:?}", acceleration.values);
        self.add(&mut acceleration);

        //println!("Vel {:?}", self.values);
        self.normalize();
    }
}
