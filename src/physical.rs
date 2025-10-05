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

use crate::entity::implementation::*;
use crate::world::*;

//================================================================

use rapier3d::{
    control::{CharacterCollision, EffectiveCharacterMovement, KinematicCharacterController},
    parry::query::ShapeCastOptions,
    prelude::*,
};
use raylib::prelude::*;
use std::sync::Arc;
use std::sync::Mutex;

//================================================================

#[derive(Default, Copy, Clone)]
pub struct Presence {
    pub rigid: RigidBodyHandle,
    pub collider: ColliderHandle,
}

impl Presence {
    /// Convenience function for creating a fixed rigid body with a cuboid collider, with point, angle, scale, and entity index already set.
    pub fn new_rigid_cuboid_fixed(
        physical: &mut Physical,
        point: Vector3,
        angle: Vector3,
        scale: Vector3,
        info: &EntityInfo,
    ) -> anyhow::Result<Self> {
        let rigid = physical.new_rigid_fixed();
        physical.set_rigid_point(rigid, point)?;
        physical.set_rigid_angle(rigid, angle)?;
        physical.set_rigid_data(rigid, info.index as u128)?;
        let collider = physical.new_cuboid(scale, Some(rigid));
        physical.set_collider_group(collider, Physical::GROUP_ENTITY)?;

        Ok(Self { rigid, collider })
    }

    /// Convenience function for creating a dynamic rigid body with a cuboid collider, with point, angle, scale, and entity index already set.
    pub fn new_rigid_cuboid_dynamic(
        physical: &mut Physical,
        point: Vector3,
        angle: Vector3,
        scale: Vector3,
        info: &EntityInfo,
    ) -> anyhow::Result<Self> {
        let rigid = physical.new_rigid_dynamic();
        physical.set_rigid_point(rigid, point)?;
        physical.set_rigid_angle(rigid, angle)?;
        physical.set_rigid_data(rigid, info.index as u128)?;
        let collider = physical.new_cuboid(scale, Some(rigid));

        Ok(Self { rigid, collider })
    }

    /// Remove this presence from the simulation. WARNING: continuing to use this Presence instance after this call WILL most likely panic!
    pub fn remove(&self, physical: &mut Physical) {
        physical.remove_rigid(self.rigid, true);
    }
}

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
    pub const GROUP_GEOMETRY: InteractionGroups =
        InteractionGroups::new(Group::GROUP_1, Group::GROUP_1);
    pub const GROUP_ENTITY: InteractionGroups =
        InteractionGroups::new(Group::GROUP_2, Group::GROUP_2);

    /// Run a tick in the physical simulation.
    pub fn tick(&mut self) {
        if let Ok(lock) = &mut self.collision_handler.collision_list.lock() {
            lock.clear();
        }

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
            &self.collision_handler,
        );
    }

    /// Draw the physical simulation's state.
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

    //================================================================

    /// Check if a point is intersecting any collider in the world.
    pub fn intersect_point(
        &self,
        point: raylib::math::Vector3,
        rigid: Option<RigidBodyHandle>,
        filter: QueryFilter,
    ) -> Option<(ColliderHandle, Collider)> {
        let filter = if let Some(rigid) = rigid {
            filter.exclude_rigid_body(rigid)
        } else {
            filter
        };

        let query_pipeline = self.broad_phase.as_query_pipeline(
            self.narrow_phase.query_dispatcher(),
            &self.rigid_body_set,
            &self.collider_set,
            filter,
        );

        if let Some(last) = query_pipeline
            .intersect_point(point![point.x, point.y, point.z])
            .last()
        {
            return Some((last.0, last.1.clone()));
        }

        None
    }

    /// Check if a cuboid is intersecting any collider in the world.
    pub fn intersect_cuboid(
        &self,
        point: Vector3,
        angle: Vector3,
        shape: Vector3,
        rigid: Option<RigidBodyHandle>,
        filter: QueryFilter,
    ) -> Option<(ColliderHandle, Collider)> {
        let filter = if let Some(rigid) = rigid {
            filter.exclude_rigid_body(rigid)
        } else {
            filter
        };

        let (v, a) = Vector4::from_euler(
            angle.y.to_radians(),
            angle.x.to_radians(),
            angle.z.to_radians(),
        )
        .to_axis_angle();
        let angle = v * a;

        let point = Isometry::new(
            vector![point.x, point.y, point.z],
            vector![angle.x, angle.y, angle.z],
        );
        let shape = Cuboid::new(vector![shape.x, shape.y, shape.z]);

        let query_pipeline = self.broad_phase.as_query_pipeline(
            self.narrow_phase.query_dispatcher(),
            &self.rigid_body_set,
            &self.collider_set,
            filter,
        );

        if let Some(last) = query_pipeline.intersect_shape(point, &shape).last() {
            return Some((last.0, last.1.clone()));
        }

        None
    }

    /// Cast a ray in the world.
    pub fn cast_ray(
        &self,
        point: Vector3,
        angle: Vector3,
        distance: f32,
        solid: bool,
        rigid: Option<RigidBodyHandle>,
        filter: QueryFilter,
    ) -> Option<(ColliderHandle, RayIntersection)> {
        let ray = rapier3d::geometry::Ray::new(
            point![point.x, point.y, point.z],
            vector![angle.x, angle.y, angle.z],
        );

        let filter = if let Some(rigid) = rigid {
            filter.exclude_rigid_body(rigid)
        } else {
            filter
        };

        let query_pipeline = self.broad_phase.as_query_pipeline(
            self.narrow_phase.query_dispatcher(),
            &self.rigid_body_set,
            &self.collider_set,
            filter,
        );

        query_pipeline.cast_ray_and_get_normal(&ray, distance, solid)
    }

    /// Cast a cuboid in the world.
    pub fn cast_cuboid(
        &self,
        point: Vector3,
        shape: Vector3,
        speed: Vector3,
        distance: f32,
        rigid: Option<RigidBodyHandle>,
        filter: QueryFilter,
    ) -> Option<(ColliderHandle, ShapeCastHit)> {
        let point = Isometry::new(vector![point.x, point.y, point.z], vector![0.0, 0.0, 0.0]);
        let shape = Cuboid::new(vector![shape.x, shape.y, shape.z]);
        let speed = vector![speed.x, speed.y, speed.z];

        let filter = if let Some(rigid) = rigid {
            filter.exclude_rigid_body(rigid)
        } else {
            filter
        };

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
    ) -> anyhow::Result<(Option<CharacterCollision>, EffectiveCharacterMovement)> {
        let wish_speed = vector![speed.x, speed.y, speed.z] * World::TIME_STEP;
        let mut collision = vec![];
        let mut collision_character = None;

        let movement = {
            let collider = self.get_collider(collider_handle)?;

            let query_pipeline = self.broad_phase.as_query_pipeline(
                self.narrow_phase.query_dispatcher(),
                &self.rigid_body_set,
                &self.collider_set,
                QueryFilter::default()
                    .exclude_collider(collider_handle)
                    .exclude_sensors(),
            );

            character.move_shape(
                World::TIME_STEP,
                &query_pipeline,
                collider.shape(),
                collider.position(),
                wish_speed,
                |event| {
                    collision_character = Some(event);
                    collision.push(event);
                },
            )
        };

        let collider = self.get_collider(collider_handle)?.clone();

        let mut query_pipeline = self.broad_phase.as_query_pipeline_mut(
            self.narrow_phase.query_dispatcher(),
            &mut self.rigid_body_set,
            &mut self.collider_set,
            QueryFilter::default()
                .exclude_collider(collider_handle)
                .exclude_sensors(),
        );

        character.solve_character_collision_impulses(
            World::TIME_STEP,
            &mut query_pipeline,
            collider.shape(),
            collider.mass(),
            &collision,
        );

        // TO-DO move this out of here
        let collider = self.get_collider_mutable(collider_handle)?;
        let position = collider.translation() + movement.translation;

        collider.set_translation(position);

        Ok((collision_character, movement))
    }

    //================================================================

    /// Create a new rigid body (fixed).
    pub fn new_rigid_fixed(&mut self) -> RigidBodyHandle {
        let rigid = RigidBodyBuilder::fixed().build();

        self.rigid_body_set.insert(rigid)
    }

    /// Create a new rigid body (dynamic).
    pub fn new_rigid_dynamic(&mut self) -> RigidBodyHandle {
        let rigid = RigidBodyBuilder::dynamic().build();

        self.rigid_body_set.insert(rigid)
    }

    /// Remove a rigid body, and optionally, any collider bound to it, from the simulation.
    pub fn remove_rigid(&mut self, handle: RigidBodyHandle, remove_collider: bool) {
        self.rigid_body_set.remove(
            handle,
            &mut self.island_manager,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            remove_collider,
        );
    }

    /// Get the actual rigid body from a handle.
    pub fn get_rigid(&self, handle: RigidBodyHandle) -> anyhow::Result<&RigidBody> {
        self.rigid_body_set
            .get(handle)
            .ok_or(anyhow::Error::msg(format!(
                "Physical::get_rigid(): Could not get rigid body \"{handle:?}\""
            )))
    }

    /// Get the actual (mutable) rigid body from a handle.
    pub fn get_rigid_mutable(&mut self, handle: RigidBodyHandle) -> anyhow::Result<&mut RigidBody> {
        self.rigid_body_set
            .get_mut(handle)
            .ok_or(anyhow::Error::msg(format!(
                "Physical::get_rigid_mutable(): Could not get rigid body \"{handle:?}\""
            )))
    }

    /// Get the point of a rigid body.
    pub fn get_rigid_point(&mut self, handle: RigidBodyHandle) -> anyhow::Result<Vector3> {
        let point = self.get_rigid(handle)?.translation();

        Ok(Vector3::new(point.x, point.y, point.z))
    }

    /// Set the point of a rigid body.
    pub fn set_rigid_point(
        &mut self,
        handle: RigidBodyHandle,
        point: Vector3,
    ) -> anyhow::Result<()> {
        self.get_rigid_mutable(handle)?
            .set_translation(vector![point.x, point.y, point.z], true);

        Ok(())
    }

    /// Get the angle of a rigid body.
    pub fn get_rigid_angle(&mut self, handle: RigidBodyHandle) -> anyhow::Result<Vector4> {
        let angle = self.get_rigid(handle)?.rotation();

        Ok(Vector4::new(angle.i, angle.j, angle.k, angle.w))
    }

    /// Set the angle of a rigid body.
    pub fn set_rigid_angle(
        &mut self,
        handle: RigidBodyHandle,
        angle: Vector3,
    ) -> anyhow::Result<()> {
        let (v, a) = Vector4::from_euler(
            angle.y.to_radians(),
            angle.x.to_radians(),
            angle.z.to_radians(),
        )
        .to_axis_angle();
        let angle = v * a;

        self.get_rigid_mutable(handle)?
            .set_rotation(Rotation::new(vector![angle.x, angle.y, angle.z]), true);

        Ok(())
    }

    /// Apply an impulse to a rigid body.
    pub fn apply_rigid_impulse(
        &mut self,
        handle: RigidBodyHandle,
        impulse: Vector3,
    ) -> anyhow::Result<()> {
        self.get_rigid_mutable(handle)?
            .apply_impulse(vector![impulse.x, impulse.y, impulse.z], true);

        Ok(())
    }

    /// Get a rigid body's user-data.
    pub fn get_rigid_data(&mut self, handle: RigidBodyHandle) -> anyhow::Result<u128> {
        Ok(self.get_rigid(handle)?.user_data)
    }

    /// Set a rigid body's user-data.
    pub fn set_rigid_data(&mut self, handle: RigidBodyHandle, data: u128) -> anyhow::Result<()> {
        self.get_rigid_mutable(handle)?.user_data = data;

        Ok(())
    }

    /// Get the transform of a rigid body.
    #[rustfmt::skip]
    pub fn get_rigid_transform(&self, handle: RigidBodyHandle) -> anyhow::Result<raylib::math::Matrix> {
        let m = self.get_rigid(handle)?.position().to_matrix();

        Ok(raylib::math::Matrix {
            m0: m.m11, m4: m.m12, m8: m.m13, m12: m.m14,
            m1: m.m21, m5: m.m22, m9: m.m23, m13: m.m24,
            m2: m.m31, m6: m.m32, m10: m.m33, m14: m.m34,
            m3: m.m41, m7: m.m42, m11: m.m43, m15: m.m44,
        })
    }

    //================================================================

    // Create a new cuboid collider.
    pub fn new_cuboid(
        &mut self,
        shape: Vector3,
        parent: Option<RigidBodyHandle>,
    ) -> ColliderHandle {
        let collider = ColliderBuilder::cuboid(shape.x, shape.y, shape.z)
            //.active_events(ActiveEvents::COLLISION_EVENTS)
            //.active_collision_types(ActiveCollisionTypes::all())
            .build();

        if let Some(parent) = parent {
            self.collider_set
                .insert_with_parent(collider, parent, &mut self.rigid_body_set)
        } else {
            self.collider_set.insert(collider)
        }
    }

    // Create a new model collider.
    pub fn new_model(
        &mut self,
        model: &Model,
        parent: Option<RigidBodyHandle>,
    ) -> anyhow::Result<()> {
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

            if let Some(parent) = parent {
                self.collider_set
                    .insert_with_parent(collider, parent, &mut self.rigid_body_set);
            } else {
                self.collider_set.insert(collider);
            }
        }

        Ok(())
    }

    /// Create a new collider.
    pub fn get_collider(&self, handle: ColliderHandle) -> anyhow::Result<&Collider> {
        self.collider_set
            .get(handle)
            .ok_or(anyhow::Error::msg(format!(
                "Physical::get_collider(): Could not get collider \"{handle:?}\""
            )))
    }

    pub fn get_collider_mutable(
        &mut self,
        handle: ColliderHandle,
    ) -> anyhow::Result<&mut Collider> {
        self.collider_set
            .get_mut(handle)
            .ok_or(anyhow::Error::msg(format!(
                "Physical::get_collider_mut(): Could not get collider \"{handle:?}\""
            )))
    }

    /// Set the point of a collider.
    pub fn set_collider_point(
        &mut self,
        handle: ColliderHandle,
        point: Vector3,
    ) -> anyhow::Result<()> {
        let handle = self.get_collider_mutable(handle)?;

        if handle.parent().is_some() {
            handle.set_translation_wrt_parent(vector![point.x, point.y, point.z]);
        } else {
            handle.set_translation(vector![point.x, point.y, point.z]);
        }

        Ok(())
    }

    /// Set a collider's sensor app.
    pub fn set_collider_sensor(
        &mut self,
        handle: ColliderHandle,
        sensor: bool,
    ) -> anyhow::Result<()> {
        self.get_collider_mutable(handle)?.set_sensor(sensor);

        Ok(())
    }

    /// Set a collider's group.
    pub fn set_collider_group(
        &mut self,
        handle: ColliderHandle,
        group: InteractionGroups,
    ) -> anyhow::Result<()> {
        self.get_collider_mutable(handle)?
            .set_collision_groups(group);

        Ok(())
    }

    /// Get a collider's user-data.
    pub fn get_collider_data(&self, handle: ColliderHandle) -> anyhow::Result<u128> {
        Ok(self.get_collider(handle)?.user_data)
    }

    /// Set a collider's user-data.
    pub fn set_collider_data(&mut self, handle: ColliderHandle, data: u128) -> anyhow::Result<()> {
        self.get_collider_mutable(handle)?.user_data = data;

        Ok(())
    }
}

//================================================================

#[derive(Default)]
pub struct CollisionHandler {
    pub collision_list: Arc<Mutex<Vec<CollisionEvent>>>,
}

impl EventHandler for CollisionHandler {
    fn handle_collision_event(
        &self,
        _: &RigidBodySet,
        _: &ColliderSet,
        event: CollisionEvent,
        _: Option<&ContactPair>,
    ) {
        if let Ok(mut lock) = self.collision_list.lock() {
            lock.push(event);
        }
    }

    fn handle_contact_force_event(
        &self,
        _: f32,
        _: &RigidBodySet,
        _: &ColliderSet,
        _: &ContactPair,
        _: f32,
    ) {
    }
}

//================================================================

struct DebugRender {}

impl DebugRenderBackend for DebugRender {
    fn draw_line(&mut self, _: DebugRenderObject, a: Point<f32>, b: Point<f32>, _: DebugColor) {
        unsafe {
            ffi::DrawLine3D(
                Vector3::new(a.x, a.y, a.z).into(),
                Vector3::new(b.x, b.y, b.z).into(),
                Color::RED.into(),
            );
        }
    }
}
