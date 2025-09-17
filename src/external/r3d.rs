#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use std::ffi::CString;

use ffi::{
    R3D_BlendMode_R3D_BLEND_ALPHA, R3D_ShadowCastMode_R3D_SHADOW_CAST_DISABLED,
    R3D_ShadowCastMode_R3D_SHADOW_CAST_ON_DOUBLE_SIDED, R3D_ShadowUpdateMode,
};
use raylib::prelude::*;
use serde::{Deserialize, Serialize};

mod ffi {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

impl Into<ffi::Vector3> for Vector3 {
    fn into(self) -> ffi::Vector3 {
        ffi::Vector3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

impl Into<BoundingBox> for ffi::BoundingBox {
    fn into(self) -> BoundingBox {
        BoundingBox {
            min: self.min.into(),
            max: self.max.into(),
        }
    }
}

impl Into<Vector3> for ffi::Vector3 {
    fn into(self) -> Vector3 {
        Vector3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

impl Into<ffi::Camera3D> for Camera3D {
    fn into(self) -> ffi::Camera3D {
        ffi::Camera3D {
            position: self.position.into(),
            target: self.target.into(),
            up: self.up.into(),
            fovy: self.fovy,
            projection: self.camera_type() as i32,
        }
    }
}

impl Into<ffi::Color> for Color {
    fn into(self) -> ffi::Color {
        ffi::Color {
            r: self.r,
            g: self.g,
            b: self.b,
            a: self.a,
        }
    }
}

impl Into<Color> for ffi::Color {
    fn into(self) -> Color {
        Color {
            r: self.r,
            g: self.g,
            b: self.b,
            a: self.a,
        }
    }
}

pub struct Handle {}

impl Handle {
    /// Initializes the rendering engine.
    ///
    /// This function sets up the internal rendering system with the provided resolution
    /// and state flags, which define the internal behavior. These flags can be modified
    /// later via set_state.
    pub fn new(resolution: (i32, i32)) -> Self {
        unsafe {
            ffi::R3D_Init(resolution.0, resolution.1, 0);
            ffi::R3D_SetTextureFilter(0);
        }

        Self {}
    }

    pub fn render<T: FnMut(&mut Self)>(&mut self, camera: Camera3D, mut call: T) {
        unsafe {
            ffi::R3D_Begin(camera.into());

            call(self);

            ffi::R3D_End();
        }
    }

    pub fn render_ex<T: FnMut(&mut Self)>(
        &mut self,
        camera: Camera3D,
        target: &mut RenderTexture2D,
        mut call: T,
    ) {
        unsafe {
            let target =
                target as *mut raylib::prelude::RenderTexture2D as *const ffi::RenderTexture;

            ffi::R3D_BeginEx(camera.into(), target);

            call(self);

            ffi::R3D_End();
        }
    }

    /// Gets the current internal resolution.
    ///
    /// This function retrieves the current internal resolution being used by the
    /// rendering engine.
    pub fn get_resolution(&self) -> (i32, i32) {
        let mut x = 0;
        let mut y = 0;

        unsafe {
            ffi::R3D_GetResolution(&mut x, &mut y);
        }

        (x, y)
    }

    /// Updates the internal resolution.
    ///
    /// This function changes the internal resolution of the rendering engine. Note that
    /// this process destroys and recreates all framebuffers, which may be a slow operation.
    pub fn update_resolution(&mut self, resolution: (i32, i32)) {
        unsafe {
            ffi::R3D_UpdateResolution(resolution.0, resolution.1);
        }
    }

    pub fn set_bloom_mode(&mut self, mode: BloomMode) {
        unsafe {
            ffi::R3D_SetBloomMode(mode as u32);
        }
    }

    pub fn set_screen_space_reflection(&mut self, enabled: bool) {
        unsafe {
            ffi::R3D_SetSSR(enabled);
        }
    }

    pub fn set_fog_mode(&mut self, mode: FogMode) {
        unsafe {
            ffi::R3D_SetFogMode(mode as u32);
        }
    }

    pub fn set_depth_of_field(&mut self, enabled: bool) {
        unsafe {
            ffi::R3D_SetDofMode(enabled as u32);
        }
    }

    pub fn is_point_in_frustum(&self, position: Vector3) -> bool {
        unsafe { ffi::R3D_IsPointInFrustum(position.into()) }
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            ffi::R3D_Close();
        }
    }
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LightType {
    #[default]
    /// Directional light, affects the entire scene with parallel rays.
    Directional,
    /// Spot light, emits light in a cone shape.
    Spot,
    /// Omni light, emits light in all directions from a single point.
    Omnidirectional,
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShadowUpdateMode {
    #[default]
    /// Shadow maps update every frame for real-time accuracy.
    Manual,
    /// Shadow maps update only when explicitly requested.
    Interval,
    /// Shadow maps update at defined time intervals.
    Continuous,
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BloomMode {
    #[default]
    /// Bloom effect is disabled. The scene is rendered without any glow enhancement.
    Disabled,
    /// Blends the bloom effect with the original scene using linear interpolation (Lerp).
    Mix,
    /// Adds the bloom effect additively to the scene, intensifying bright regions.
    Additive,
    /// Combines the scene and bloom using screen blending, which brightens highlights.
    Screen,
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FogMode {
    #[default]
    /// Fog effect is disabled.
    Disabled,
    /// Fog density increases linearly with distance from the camera.
    Linear,
    /// Exponential fog (exp2), where density increases exponentially with distance.
    Exponential2,
    /// Exponential fog, similar to EXP2 but with a different rate of increase.
    Exponential,
}

pub struct Light {
    inner: ffi::R3D_Light,
}

impl Light {
    pub fn new(_handle: &mut Handle, light_type: LightType) -> Self {
        let inner = unsafe { ffi::R3D_CreateLight(light_type as u32) };

        Self { inner }
    }

    pub fn get_type(&self) -> LightType {
        unsafe {
            match ffi::R3D_GetLightType(self.inner) {
                0 => LightType::Directional,
                1 => LightType::Spot,
                _ => LightType::Omnidirectional,
            }
        }
    }

    pub fn is_active(&self) -> bool {
        unsafe { ffi::R3D_IsLightActive(self.inner) }
    }

    pub fn toggle(&mut self) {
        unsafe { ffi::R3D_ToggleLight(self.inner) }
    }

    pub fn set_active(&mut self, active: bool) {
        unsafe {
            ffi::R3D_SetLightActive(self.inner, active);
        }
    }

    pub fn get_color(&self) -> Color {
        unsafe { ffi::R3D_GetLightColor(self.inner).into() }
    }

    pub fn get_color_vector(&self) -> Vector3 {
        unsafe { ffi::R3D_GetLightColorV(self.inner).into() }
    }

    pub fn set_color(&mut self, color: Color) {
        unsafe {
            ffi::R3D_SetLightColor(self.inner, color.into());
        }
    }

    pub fn set_color_vector(&mut self, color: Vector3) {
        unsafe {
            ffi::R3D_SetLightColorV(self.inner, color.into());
        }
    }

    pub fn get_position(&self) -> Option<Vector3> {
        unsafe {
            if self.get_type() == LightType::Directional {
                None
            } else {
                Some(ffi::R3D_GetLightPosition(self.inner).into())
            }
        }
    }

    pub fn set_position(&mut self, position: Vector3) -> Result<(), String> {
        unsafe {
            if self.get_type() == LightType::Directional {
                Err("Can't set position on a directional light source.".to_string())
            } else {
                ffi::R3D_SetLightPosition(self.inner, position.into());
                Ok(())
            }
        }
    }

    pub fn get_direction(&self) -> Option<Vector3> {
        unsafe {
            if self.get_type() == LightType::Omnidirectional {
                None
            } else {
                Some(ffi::R3D_GetLightDirection(self.inner).into())
            }
        }
    }

    pub fn set_direction(&mut self, direction: Vector3) -> Result<(), String> {
        unsafe {
            if self.get_type() == LightType::Omnidirectional {
                Err("Can't set direction on an omni-directional light source.".to_string())
            } else {
                ffi::R3D_SetLightDirection(self.inner, direction.into());
                Ok(())
            }
        }
    }

    pub fn look_at(&mut self, position: Vector3, target: Vector3) {
        unsafe {
            ffi::R3D_LightLookAt(self.inner, position.into(), target.into());
        }
    }

    pub fn get_energy(&self) -> f32 {
        unsafe { ffi::R3D_GetLightEnergy(self.inner) }
    }

    pub fn set_energy(&mut self, energy: f32) {
        unsafe { ffi::R3D_SetLightEnergy(self.inner, energy) }
    }

    pub fn get_specular(&self) -> f32 {
        unsafe { ffi::R3D_GetLightSpecular(self.inner) }
    }

    pub fn set_specular(&mut self, specular: f32) {
        unsafe { ffi::R3D_SetLightSpecular(self.inner, specular) }
    }

    pub fn get_range(&self) -> f32 {
        unsafe { ffi::R3D_GetLightRange(self.inner) }
    }

    pub fn set_range(&mut self, range: f32) {
        unsafe { ffi::R3D_SetLightRange(self.inner, range) }
    }

    pub fn get_attenuation(&self) -> f32 {
        unsafe { ffi::R3D_GetLightAttenuation(self.inner) }
    }

    pub fn set_attenuation(&mut self, attenuation: f32) {
        unsafe { ffi::R3D_SetLightAttenuation(self.inner, attenuation) }
    }

    pub fn get_inner_cut_off(&self) -> f32 {
        unsafe { ffi::R3D_GetLightInnerCutOff(self.inner) }
    }

    pub fn set_inner_cut_off(&mut self, degrees: f32) {
        unsafe { ffi::R3D_SetLightInnerCutOff(self.inner, degrees) }
    }

    pub fn get_outer_cut_off(&self) -> f32 {
        unsafe { ffi::R3D_GetLightOuterCutOff(self.inner) }
    }

    pub fn set_outer_cut_off(&mut self, degrees: f32) {
        unsafe { ffi::R3D_SetLightOuterCutOff(self.inner, degrees) }
    }

    pub fn enable_shadow(&mut self, resolution: i32) {
        unsafe {
            ffi::R3D_EnableShadow(self.inner, resolution);
        }
    }

    pub fn disable_shadow(&mut self, destroy_map: bool) {
        unsafe {
            ffi::R3D_DisableShadow(self.inner, destroy_map);
        }
    }

    pub fn is_shadow_enabled(&self) -> bool {
        unsafe { ffi::R3D_IsShadowEnabled(self.inner) }
    }

    pub fn has_shadow_map(&self) -> bool {
        unsafe { ffi::R3D_HasShadowMap(self.inner) }
    }

    pub fn get_shadow_update_mode(&self) -> ShadowUpdateMode {
        unsafe {
            match ffi::R3D_GetShadowUpdateMode(self.inner) {
                0 => ShadowUpdateMode::Manual,
                1 => ShadowUpdateMode::Interval,
                _ => ShadowUpdateMode::Continuous,
            }
        }
    }

    pub fn set_shadow_update_mode(&mut self, mode: ShadowUpdateMode) {
        unsafe {
            ffi::R3D_SetShadowUpdateMode(self.inner, mode as u32);
        }
    }

    pub fn get_shadow_update_frequency(&self) -> i32 {
        unsafe { ffi::R3D_GetShadowUpdateFrequency(self.inner) }
    }

    pub fn set_shadow_update_frequency(&mut self, millisecond: i32) {
        unsafe {
            ffi::R3D_SetShadowUpdateFrequency(self.inner, millisecond);
        }
    }

    pub fn update_shadow_map(&mut self) {
        unsafe {
            ffi::R3D_UpdateShadowMap(self.inner);
        }
    }

    pub fn get_shadow_softness(&self) -> f32 {
        unsafe { ffi::R3D_GetShadowSoftness(self.inner) }
    }

    pub fn set_shadow_softness(&mut self, softness: f32) {
        unsafe {
            ffi::R3D_SetShadowSoftness(self.inner, softness);
        }
    }

    pub fn get_shadow_depth_bias(&self) -> f32 {
        unsafe { ffi::R3D_GetShadowDepthBias(self.inner) }
    }

    pub fn set_shadow_depth_bias(&mut self, value: f32) {
        unsafe {
            ffi::R3D_SetShadowDepthBias(self.inner, value);
        }
    }

    pub fn get_shadow_slope_bias(&self) -> f32 {
        unsafe { ffi::R3D_GetShadowSlopeBias(self.inner) }
    }

    pub fn set_shadow_slope_bias(&mut self, value: f32) {
        unsafe {
            ffi::R3D_SetShadowSlopeBias(self.inner, value);
        }
    }
}

impl Drop for Light {
    fn drop(&mut self) {
        unsafe {
            ffi::R3D_DestroyLight(self.inner);
        }
    }
}

pub struct Vertex {
    inner: ffi::R3D_Vertex,
}

impl Vertex {
    fn from_raw(raw: ffi::R3D_Vertex) -> Self {
        Self { inner: raw }
    }

    pub fn position(&self) -> Vector3 {
        self.inner.position.into()
    }
}

pub struct Mesh {
    inner: ffi::R3D_Mesh,
    weak: bool,
}

impl Mesh {
    pub fn generate_cube(
        handle: &mut Handle,
        width: f32,
        height: f32,
        length: f32,
        upload: bool,
    ) -> Self {
        let inner = unsafe { ffi::R3D_GenMeshCube(width, height, length, upload) };

        Self { inner, weak: false }
    }

    fn from_raw_weak(raw: ffi::R3D_Mesh) -> Self {
        Self {
            inner: raw,
            weak: true,
        }
    }

    pub fn vertices(&self) -> Vec<Vertex> {
        unsafe {
            let vector: Vec<Vertex> = std::slice::from_raw_parts(
                self.inner.vertices as *const ffi::R3D_Vertex,
                self.inner.vertexCount as usize,
            )
            .iter()
            .map(|x| Vertex::from_raw(*x))
            .collect();

            vector
        }
    }

    pub fn indicies(&self) -> &[u32] {
        unsafe {
            std::slice::from_raw_parts(
                self.inner.indices as *const u32,
                self.inner.indexCount as usize,
            )
        }
    }

    pub fn vertex_count(&self) -> i32 {
        self.inner.vertexCount as i32
    }

    pub fn index_count(&self) -> i32 {
        self.inner.indexCount as i32
    }
}

impl Drop for Mesh {
    fn drop(&mut self) {
        if !self.weak {
            unsafe {
                ffi::R3D_UnloadMesh(&self.inner);
            }
        }
    }
}

pub struct Model {
    inner: ffi::R3D_Model,
    weak: bool,
}

impl Model {
    pub fn load_from_mesh(handle: &mut Handle, mesh: &mut Mesh) -> Self {
        mesh.weak = true;

        let inner = unsafe { ffi::R3D_LoadModelFromMesh(&mesh.inner) };

        Self { inner, weak: false }
    }

    pub fn new(_handle: &mut Handle, file_path: &str) -> Self {
        let inner = unsafe { ffi::R3D_LoadModel(CString::new(file_path).unwrap().as_ptr()) };

        unsafe {
            /*
            for x in 0..inner.materialCount {
                let mut material = *inner.materials.wrapping_add(x as usize);

                material.albedo.color = Color::new(255, 255, 255, 127).into();
                material.orm.occlusion = 0.0;
                material.orm.roughness = 0.0;
                material.orm.metalness = 0.0;
                material.blendMode = R3D_BlendMode_R3D_BLEND_ALPHA;

                *inner.materials.wrapping_add(x as usize) = material;
            }
            */
        }

        Self { inner, weak: false }
    }

    fn weak(&self) -> Self {
        Self {
            inner: self.inner,
            weak: true,
        }
    }

    pub fn draw(&self, _handle: &mut Handle, position: Vector3, scale: f32) {
        unsafe {
            ffi::R3D_DrawModel(&self.inner, position.into(), scale);
        }
    }

    pub fn draw_ex(
        &self,
        _handle: &mut Handle,
        position: Vector3,
        rotationAxis: Vector3,
        rotationAngle: f32,
        scale: Vector3,
    ) {
        unsafe {
            ffi::R3D_DrawModelEx(
                &self.inner,
                position.into(),
                rotationAxis.into(),
                rotationAngle,
                scale.into(),
            );
        }
    }

    pub fn bounding_box(&self) -> BoundingBox {
        self.inner.aabb.into()
    }

    // TO-DO this should not use a vector...
    pub fn meshes(&self) -> Vec<Mesh> {
        unsafe {
            let vector: Vec<Mesh> = std::slice::from_raw_parts(
                self.inner.meshes as *const ffi::R3D_Mesh,
                self.inner.meshCount as usize,
            )
            .iter()
            .map(|x| Mesh::from_raw_weak(*x))
            .collect();

            vector
        }
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        unsafe {
            if !self.weak {
                ffi::R3D_UnloadModel(&self.inner, true);
            }
        }
    }
}
