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
    parry::query::{QueryDispatcher, ShapeCastOptions},
    prelude::*,
};
use raylib::prelude::*;
use std::sync::Arc;
use std::sync::Mutex;

//================================================================

#[derive(Default)]
pub struct Physical {
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    debug_render_pipeline: DebugRenderPipeline,
    pub collision_handler: CollisionHandler,
}

impl Physical {
    pub fn new_model(&mut self, model: &crate::external::r3d::Model) -> anyhow::Result<()> {
        for mesh in model.meshes() {
            let list_vertex = mesh
                .vertices()
                .iter()
                .map(|v| point![v.position().x, v.position().y, v.position().z])
                .collect();
            let mut list_index = Vec::new();
            let index = mesh.indicies();

            for x in 0..index.len() / 3 {
                list_index.push([
                    index[x * 3] as u32,
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

        let query_pipeline = self.broad_phase.as_query_pipeline(
            self.narrow_phase.query_dispatcher(),
            &self.rigid_body_set,
            &self.collider_set,
            filter,
        );

        query_pipeline.cast_ray_and_get_normal(&ray, distance, solid)
    }

    pub fn cast_cuboid(
        &self,
        point: Vector3,
        shape: Vector3,
        speed: Vector3,
        distance: f32,
        filter: QueryFilter,
    ) -> Option<(ColliderHandle, ShapeCastHit)> {
        let point = Isometry::new(vector![point.x, point.y, point.z], vector![0.0, 0.0, 0.0]);
        let shape = Cuboid::new(vector![shape.x, shape.y, shape.z]);
        let speed = vector![speed.x, speed.y, speed.z];

        let query_pipeline = self.broad_phase.as_query_pipeline(
            self.narrow_phase.query_dispatcher(),
            &self.rigid_body_set,
            &self.collider_set,
            filter,
        );

        query_pipeline.cast_shape(
            &point,
            &speed,
            &shape,
            ShapeCastOptions {
                max_time_of_impact: distance,
                target_distance: 0.0,
                stop_at_penetration: false,
                compute_impact_geometry_on_penetration: false,
            },
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

        let query_pipeline = self.broad_phase.as_query_pipeline(
            self.narrow_phase.query_dispatcher(),
            &self.rigid_body_set,
            &self.collider_set,
            QueryFilter::default()
                .exclude_collider(collider_handle)
                .exclude_sensors(),
        );

        let movement = character.move_shape(
            World::TIME_STEP,
            &query_pipeline,
            collider.shape(),
            collider.position(),
            wish_speed,
            |_| {},
        );

        let collider = self.get_collider_mut(collider_handle)?;
        let position = collider.translation() + movement.translation;

        collider.set_translation(position);

        Ok(movement)
    }

    pub fn new_cuboid_entity(
        &mut self,
        point: Vector3,
        shape: Vector3,
        index: usize,
    ) -> ColliderHandle {
        let collider = ColliderBuilder::cuboid(shape.x, shape.y, shape.z)
            .translation(vector![point.x, point.y, point.z])
            .user_data(index as u128)
            //.active_events(ActiveEvents::COLLISION_EVENTS)
            //.active_collision_types(ActiveCollisionTypes::all())
            .build();

        self.collider_set.insert(collider)
    }

    pub fn new_cuboid(&mut self, shape: Vector3) -> ColliderHandle {
        let collider = ColliderBuilder::cuboid(shape.x, shape.y, shape.z)
            //.active_events(ActiveEvents::COLLISION_EVENTS)
            //.active_collision_types(ActiveCollisionTypes::all())
            .build();

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

    pub fn set_collider_point(
        &mut self,
        handle: ColliderHandle,
        point: Vector3,
    ) -> anyhow::Result<()> {
        let handle = self.get_collider_mut(handle)?;
        handle.set_translation(vector![point.x, point.y, point.z]);

        Ok(())
    }

    pub fn set_collider_sensor(
        &mut self,
        handle: ColliderHandle,
        sensor: bool,
    ) -> anyhow::Result<()> {
        let handle = self.get_collider_mut(handle)?;
        handle.set_sensor(sensor);

        Ok(())
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
        //if let Ok(lock) = &mut self.collision_handler.collision_list.lock() {
        //    lock.clear();
        //}

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
            &(),
            &(),
            //&self.collision_handler,
        );
    }
}

#[derive(Default, Debug)]
pub struct HitEvent {
    pub collider_a: ColliderHandle,
    pub collider_b: ColliderHandle,
}

#[derive(Default)]
pub struct CollisionHandler {
    pub collision_list: Arc<Mutex<Vec<HitEvent>>>,
}

impl EventHandler for CollisionHandler {
    fn handle_collision_event(
        &self,
        _bodies: &RigidBodySet,
        _colliders: &ColliderSet,
        event: CollisionEvent,
        _contact_pair: Option<&ContactPair>,
    ) {
        if let Ok(mut lock) = self.collision_list.lock() {
            match event {
                CollisionEvent::Started(
                    collider_handle,
                    collider_handle1,
                    collision_event_flags,
                ) => lock.push(HitEvent {
                    collider_a: collider_handle,
                    collider_b: collider_handle1,
                }),
                CollisionEvent::Stopped(
                    collider_handle,
                    collider_handle1,
                    collision_event_flags,
                ) => lock.push(HitEvent {
                    collider_a: collider_handle,
                    collider_b: collider_handle1,
                }),
            }
        }
    }

    fn handle_contact_force_event(
        &self,
        _dt: f32,
        _bodies: &RigidBodySet,
        _colliders: &ColliderSet,
        _contact_pair: &ContactPair,
        _total_force_magnitude: f32,
    ) {
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
