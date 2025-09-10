/*
* Copyright (c) 2025 luxreduxdelux
*
* Redistribution and use in source and binary forms, with or without
* modification, are permitted provided that the following conditions are met:
*
* 1. Redistributions of source code must retain the above copyright notice,
* this list of conditions and the following disclaimer.
*
* 2. Redistributions in binary form must reproduce the above copyright notice,
* this list of conditions and the following disclaimer in the documentation
* and/or other materials provided with the distribution.
*
* Subject to the terms and conditions of this license, each copyright holder
* and contributor hereby grants to those receiving rights under this license
* a perpetual, worldwide, non-exclusive, no-charge, royalty-free, irrevocable
* (except for failure to satisfy the conditions of this license) patent license
* to make, have made, use, offer to sell, sell, import, and otherwise transfer
* this software, where such license applies only to those patent claims, already
* acquired or hereafter acquired, licensable by such copyright holder or
* contributor that are necessarily infringed by:
*
* (a) their Contribution(s) (the licensed copyrights of copyright holders and
* non-copyrightable additions of contributors, in source or binary form) alone;
* or
*
* (b) combination of their Contribution(s) with the work of authorship to which
* such Contribution(s) was added by such copyright holder or contributor, if,
* at the time the Contribution is added, such addition causes such combination
* to be necessarily infringed. The patent license shall not apply to any other
* combinations which include the Contribution.
*
* Except as expressly stated above, no rights or licenses from any copyright
* holder or contributor is granted under this license, whether expressly, by
* implication, estoppel or otherwise.
*
* DISCLAIMER
*
* THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
* AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
* IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
* DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDERS OR CONTRIBUTORS BE LIABLE
* FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
* DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
* SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
* CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
* OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
* OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
*/

use crate::world::*;

//================================================================

use rapier3d::{
    control::{EffectiveCharacterMovement, KinematicCharacterController},
    prelude::*,
};
use raylib::prelude::*;

//================================================================

#[derive(Default)]
pub struct Physical {
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    pub integration_parameters: IntegrationParameters,
    pub physics_pipeline: PhysicsPipeline,
    pub island_manager: IslandManager,
    pub broad_phase: DefaultBroadPhase,
    pub narrow_phase: NarrowPhase,
    pub impulse_joint_set: ImpulseJointSet,
    pub multibody_joint_set: MultibodyJointSet,
    pub ccd_solver: CCDSolver,
    pub query_pipeline: QueryPipeline,
    pub debug_render_pipeline: DebugRenderPipeline,
}

impl Physical {
    pub fn new_model(&mut self, model: &Model) -> anyhow::Result<()> {
        for mesh in model.meshes() {
            let list_vertex = mesh
                .vertices()
                .iter()
                .map(|v| point![v.x, v.y, v.z])
                .collect();
            let mut list_index = Vec::new();
            let index = unsafe {
                std::slice::from_raw_parts(
                    mesh.indices as *const u16,
                    mesh.triangleCount as usize * 3,
                )
            };

            for x in 0..index.len() / 3 {
                list_index.push([
                    index[x * 3 + 0] as u32,
                    index[x * 3 + 1] as u32,
                    index[x * 3 + 2] as u32,
                ]);
            }

            let collider = ColliderBuilder::trimesh(list_vertex, list_index)?;

            self.collider_set.insert(collider);
        }

        Ok(())
    }

    pub fn cast_ray(
        &self,
        ray: raylib::math::Ray,
        distance: f32,
        solid: bool,
        filter: QueryFilter,
    ) -> Option<(ColliderHandle, RayIntersection)> {
        let ray = rapier3d::geometry::Ray::new(
            point![ray.position.x, ray.position.y, ray.position.z],
            vector![ray.direction.x, ray.direction.y, ray.direction.z],
        );

        self.query_pipeline.cast_ray_and_get_normal(
            &self.rigid_body_set,
            &self.collider_set,
            &ray,
            distance,
            solid,
            filter,
        )
    }

    pub fn move_controller(
        &mut self,
        collider_handle: ColliderHandle,
        character: KinematicCharacterController,
        speed: Vector3,
    ) -> anyhow::Result<EffectiveCharacterMovement> {
        let wish_speed = vector![speed.x, speed.y, speed.z] * World::TIME_STEP;
        let collider = self.get_collider(collider_handle)?;

        let movement = character.move_shape(
            World::TIME_STEP,
            &self.rigid_body_set,
            &self.collider_set,
            &self.query_pipeline,
            collider.shape(),
            collider.position(),
            wish_speed,
            QueryFilter::default().exclude_collider(collider_handle),
            |_| {},
        );

        let collider = self.get_collider_mut(collider_handle)?;
        let position = collider.translation() + movement.translation;

        collider.set_translation(position);

        Ok(movement)
    }

    pub fn new_cuboid(&mut self, shape: Vector3) -> ColliderHandle {
        let collider = ColliderBuilder::cuboid(shape.x, shape.y, shape.z).build();

        self.collider_set.insert(collider)
    }

    pub fn get_collider(&self, handle: ColliderHandle) -> anyhow::Result<&Collider> {
        self.collider_set
            .get(handle)
            .ok_or(anyhow::Error::msg(format!(
                "Physical::get_collider(): Could not get collider \"{handle:?}\""
            )))
    }

    pub fn get_collider_mut(&mut self, handle: ColliderHandle) -> anyhow::Result<&mut Collider> {
        self.collider_set
            .get_mut(handle)
            .ok_or(anyhow::Error::msg(format!(
                "Physical::get_collider_mut(): Could not get collider \"{handle:?}\""
            )))
    }

    pub fn draw(&mut self) {
        self.debug_render_pipeline.render(
            &mut DebugRender {},
            &self.rigid_body_set,
            &self.collider_set,
            &self.impulse_joint_set,
            &self.multibody_joint_set,
            &self.narrow_phase,
        );
    }

    pub fn tick(&mut self) {
        self.physics_pipeline.step(
            &vector![0.0, -9.81, 0.0],
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &(),
            &(),
        );
    }
}

struct DebugRender {}

impl DebugRenderBackend for DebugRender {
    fn draw_line(
        &mut self,
        _object: DebugRenderObject,
        a: Point<f32>,
        b: Point<f32>,
        _color: DebugColor,
    ) {
        unsafe {
            ffi::DrawLine3D(
                Vector3::new(a.x, a.y, a.z).into(),
                Vector3::new(b.x, b.y, b.z).into(),
                Color::RED.into(),
            );
        }
    }
}
