#[cfg(feature = "cuda")]
extern crate rustacuda_core;

#[cfg(feature = "use_serde")]
use serde::{Deserialize, Serialize};

use crate::{Matrix, MatrixOps, Quaternion, Tuple, Tuple4D};

#[derive(Clone, Debug)]
#[cfg_attr(feature = "cuda", derive(DeviceCopy))]
#[cfg_attr(feature = "use_serde", derive(Deserialize, Serialize))]
pub struct Transform {
    pos: Tuple4D,
    rot: Quaternion,
    scale: Tuple4D,
}

impl Transform {
    pub fn new() -> Transform {
        Transform::new_from_vector(Tuple4D::new_vector(0.0, 0.0, 0.0))
    }

    pub fn new_from_vector(pos: Tuple4D) -> Transform {
        Transform::new_from_all(
            pos,
            Quaternion::new(0.0, 0.0, 0.0, 1.0),
            Tuple4D::new(1.0, 1.0, 1.0, 1.0),
        )
    }

    pub fn new_from_all(pos: Tuple4D, rot: Quaternion, scale: Tuple4D) -> Transform {
        Transform { pos, rot, scale }
    }

    pub fn rotate(&self, r: Quaternion) -> Transform {
        let rot = (r * self.rot).normalized();
        Transform {
            pos: self.pos,
            rot,
            scale: self.scale,
        }
    }

    pub fn look_at(&self, point: Tuple4D, up: Tuple4D) -> Transform {
        self.rotate(self.get_look_at_rotation(point, up))
    }

    pub fn get_look_at_rotation(&self, point: Tuple4D, up: Tuple4D) -> Quaternion {
        let p = Tuple4D::normalize(&(point - self.pos));
        let m = Matrix::init_rotation_from_forward_up(p, up);
        Quaternion::new_from_rot_matrix(m)
    }

    pub fn get_transformation(&self) -> Matrix {
        let translation_matrix = Matrix::init_translation(self.pos.get_x(), self.pos.get_y(), self.pos.get_z());
        let rotation_matrix = self.rot.to_rotation_matrix();
        let scale_matrix = Matrix::init_scale(self.scale.get_x(), self.scale.get_y(), self.scale.get_z());

        translation_matrix * (rotation_matrix * scale_matrix)
    }

    pub fn set_pos(&self, pos: Tuple4D) -> Transform {
        Transform::new_from_all(pos, self.rot, self.scale)
    }

    pub fn pos(&self) -> &Tuple4D {
        &self.pos
    }

    pub fn rot(&self) -> &Quaternion {
        &self.rot
    }

    pub fn scale(&self) -> &Tuple4D {
        &self.scale
    }
}