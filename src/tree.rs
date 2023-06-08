use bevy::prelude::{Transform, Vec3};

use crate::params::Params;

pub struct Branch(pub Transform, pub Option<usize>, pub bool);

fn generate_leaves(parent_idx: usize, all: &mut Vec<Branch>) {
    let mut child_transform = Transform::IDENTITY;
    child_transform = child_transform.with_translation(child_transform.local_y());
    all.push(Branch(child_transform, Some(parent_idx), true));
}

fn generate_branches(params: &Params, level: u8, parent_idx: usize, all: &mut Vec<Branch>) {
    assert!(level >= 1);
    assert!(params.children > 1);
    for i in 0..params.children {
        let angle_from_root_branch = params.angle_from_parent_branch;
        let child_gap_f32 = f32::from(i) / f32::from(params.children);
        let angle_around_root_branch = 2.0 * std::f32::consts::PI * child_gap_f32;
        let child_idx_f32 = f32::from(i) / f32::from(params.children - 1);

        let translation_along_root =
            (1.0 - child_idx_f32) * params.child_translation_factor + child_idx_f32;

        let mut child_transform = Transform::IDENTITY;
        child_transform.rotate_local_y(angle_around_root_branch);
        child_transform = child_transform.with_translation(
            child_transform.local_z()
                * (params.base_radius + params.child_scale * 0.5 * angle_from_root_branch.sin())
                + child_transform.local_y()
                    * ((translation_along_root - 0.5)
                        + params.child_scale * 0.5 * angle_from_root_branch.cos()),
        );
        child_transform = child_transform.with_scale(Vec3::splat(params.child_scale));
        child_transform.rotate_local_x(angle_from_root_branch);

        let child_idx = all.len();
        all.push(Branch(child_transform, Some(parent_idx), false));
        if level < params.levels {
            generate_branches(params, level + 1, child_idx, all);
        } else {
            generate_leaves(child_idx, all);
        }
    }
}

pub fn generate(params: &Params) -> Vec<Branch> {
    let base = Transform::default();
    let mut ret: Vec<Branch> = Vec::new();
    ret.push(Branch(base, None, false));
    generate_branches(params, 1, 0, &mut ret);
    ret
}
