#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use std::ffi::CString;

use raylib::prelude::*;
use serde::{Deserialize, Serialize};

mod ffi {
    include!(concat!(env!("OUT_DIR"), "/r3d_bind.rs"));
}

impl From<Vector3> for ffi::Vector3 {
    fn from(val: Vector3) -> Self {
        unsafe { std::mem::transmute(val) }
    }
}

impl From<Matrix> for ffi::Matrix {
    fn from(val: Matrix) -> Self {
        unsafe { std::mem::transmute(val) }
    }
}

impl From<ffi::Matrix> for Matrix {
    fn from(val: ffi::Matrix) -> Self {
        unsafe { std::mem::transmute(val) }
    }
}

impl From<Transform> for ffi::Transform {
    fn from(val: Transform) -> Self {
        unsafe { std::mem::transmute(val) }
    }
}

impl From<ffi::Transform> for Transform {
    fn from(val: ffi::Transform) -> Self {
        unsafe { std::mem::transmute(val) }
    }
}

impl From<ffi::BoundingBox> for BoundingBox {
    fn from(val: ffi::BoundingBox) -> Self {
        unsafe { std::mem::transmute(val) }
    }
}

impl From<BoundingBox> for ffi::BoundingBox {
    fn from(val: BoundingBox) -> Self {
        unsafe { std::mem::transmute(val) }
    }
}

impl From<ffi::Vector3> for Vector3 {
    fn from(val: ffi::Vector3) -> Self {
        unsafe { std::mem::transmute(val) }
    }
}

impl From<Camera3D> for ffi::Camera3D {
    fn from(val: Camera3D) -> Self {
        ffi::Camera3D {
            position: val.position.into(),
            target: val.target.into(),
            up: val.up.into(),
            fovy: val.fovy,
            projection: val.camera_type() as i32,
        }
    }
}

impl From<Color> for ffi::Color {
    fn from(val: Color) -> Self {
        unsafe { std::mem::transmute(val) }
    }
}

impl From<ffi::Color> for Color {
    fn from(val: ffi::Color) -> Self {
        unsafe { std::mem::transmute(val) }
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
            let flag = ffi::R3D_FLAG_BLIT_LINEAR
                | ffi::R3D_FLAG_FORCE_FORWARD
                | ffi::R3D_FLAG_DEPTH_PREPASS
                | ffi::R3D_FLAG_8_BIT_NORMALS
                | ffi::R3D_FLAG_LOW_PRECISION_BUFFERS;

            ffi::R3D_Init(resolution.0, resolution.1, flag);
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

    pub fn is_bounding_box_in_frustum(&self, bounding_box: BoundingBox) -> bool {
        unsafe { ffi::R3D_IsAABBInFrustum(bounding_box.into()) }
    }

    pub fn get_brightness(&self) -> f32 {
        unsafe { ffi::R3D_GetBrightness() }
    }

    pub fn set_brightness(&self, value: f32) {
        unsafe {
            ffi::R3D_SetBrightness(value);
        }
    }

    pub fn get_contrast(&self) -> f32 {
        unsafe { ffi::R3D_GetContrast() }
    }

    pub fn set_contrast(&self, value: f32) {
        unsafe {
            ffi::R3D_SetContrast(value);
        }
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
    /// Shadow maps update only when explicitly requested.
    Manual,
    /// Shadow maps update at defined time intervals.
    Interval,
    /// Shadow maps update every frame for real-time accuracy.
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

    pub fn get_bounding_box(&self) -> BoundingBox {
        unsafe { ffi::R3D_GetLightBoundingBox(self.inner).into() }
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

// shouldn't inner be a reference?
pub struct Mesh {
    inner: ffi::R3D_Mesh,
    weak: bool,
}

impl Mesh {
    pub fn generate_cube(
        _handle: &mut Handle,
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
        self.inner.vertexCount
    }

    pub fn index_count(&self) -> i32 {
        self.inner.indexCount
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
    pub fn load_from_mesh(_handle: &mut Handle, mesh: &mut Mesh) -> Self {
        mesh.weak = true;

        let inner = unsafe { ffi::R3D_LoadModelFromMesh(&mesh.inner) };

        Self { inner, weak: false }
    }

    pub fn new(_handle: &mut Handle, file_path: &str) -> Self {
        let inner = unsafe { ffi::R3D_LoadModel(CString::new(file_path).unwrap().as_ptr()) };

        unsafe {
            if false {
                for x in 0..inner.materialCount {
                    let mut material = *inner.materials.wrapping_add(x as usize);

                    material.albedo.color = Color::new(255, 255, 255, 127).into();
                    material.orm.occlusion = 0.0;
                    material.orm.roughness = 0.0;
                    material.orm.metalness = 0.0;
                    material.blendMode = ffi::R3D_BlendMode_R3D_BLEND_ALPHA;

                    *inner.materials.wrapping_add(x as usize) = material;
                }
            }
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

    pub fn draw_pro(&self, _handle: &mut Handle, transform: Matrix) {
        unsafe {
            ffi::R3D_DrawModelPro(&self.inner, transform.into());
        }
    }

    pub fn material_count(&self) -> i32 {
        self.inner.materialCount
    }

    pub fn get_bounding_box(&self) -> BoundingBox {
        self.inner.aabb.into()
    }

    pub fn get_bone_offsets(&self) -> Vec<Matrix> {
        unsafe {
            let vector: Vec<Matrix> = std::slice::from_raw_parts(
                self.inner.boneOffsets as *const ffi::Matrix,
                self.inner.boneCount as usize,
            )
            .iter()
            .map(|x| (*x).into())
            .collect();

            vector
        }
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

    pub fn materials(&mut self) -> Vec<Material> {
        unsafe {
            let vector: Vec<Material> = std::slice::from_raw_parts_mut(
                self.inner.materials,
                self.inner.materialCount as usize,
            )
            .iter_mut()
            .map(|x| Material::from_raw_weak(x))
            .collect();

            vector
        }
    }

    pub fn get_model_animation(&self) -> Option<ModelAnimation> {
        if self.inner.anim.is_null() {
            None
        } else {
            Some(ModelAnimation {
                inner: self.inner.anim,
            })
        }
    }

    pub fn set_model_animation(&mut self, model_animation: Option<ModelAnimation>) {
        if let Some(m_a) = model_animation {
            self.inner.anim = m_a.inner;
        } else {
            self.inner.anim = std::ptr::null();
        }
    }

    pub fn get_animation_frame(&self) -> i32 {
        self.inner.animFrame
    }

    pub fn set_animation_frame(&mut self, frame: i32) {
        self.inner.animFrame = frame;
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

pub struct ModelAnimation {
    inner: *const ffi::R3D_ModelAnimation,
}

impl ModelAnimation {
    pub fn get_frame_count(&self) -> i32 {
        unsafe { (*self.inner).frameCount }
    }

    pub fn get_bone_info(&self) -> &[BoneInfo] {
        unsafe {
            std::slice::from_raw_parts(
                (*self.inner).bones as *const BoneInfo,
                (*self.inner).boneCount as usize,
            )
        }
    }

    pub fn get_frame_global_poses(&self, frame: usize) -> Vec<Matrix> {
        unsafe {
            let array = (*self.inner).frameGlobalPoses;
            let frame = *(array.wrapping_add(frame));

            let vector: Vec<Matrix> = std::slice::from_raw_parts(
                frame as *const ffi::Matrix,
                (*self.inner).boneCount as usize,
            )
            .iter()
            .map(|x| (*x).into())
            .collect();

            vector
        }
    }

    pub fn get_frame_local_poses(&self, frame: usize) -> Vec<Transform> {
        unsafe {
            let array = (*self.inner).frameLocalPoses;
            let frame = *(array.wrapping_add(frame));

            let vector: Vec<Transform> = std::slice::from_raw_parts(
                frame as *const ffi::Transform,
                (*self.inner).boneCount as usize,
            )
            .iter()
            .map(|x| (*x).into())
            .collect();

            vector
        }
    }
}

pub struct ModelAnimationList {
    inner: *mut ffi::R3D_ModelAnimation,
    count: i32,
}

impl ModelAnimationList {
    pub fn new(
        _handle: &mut Handle,
        file_path: &str,
        target_frame_rate: i32,
    ) -> ModelAnimationList {
        let mut count = 0;
        let inner = unsafe {
            ffi::R3D_LoadModelAnimations(
                rust_to_c_string(file_path).as_ptr(),
                &mut count,
                target_frame_rate,
            )
        };

        Self { inner, count }
    }

    pub fn get_animation(&self, name: &str) -> Option<ModelAnimation> {
        let inner = unsafe {
            ffi::R3D_GetModelAnimation(self.inner, self.count, rust_to_c_string(name).as_ptr())
        };

        if inner.is_null() {
            None
        } else {
            Some(ModelAnimation { inner })
        }
    }
}

impl Drop for ModelAnimationList {
    fn drop(&mut self) {
        unsafe {
            ffi::R3D_UnloadModelAnimations(self.inner, self.count);
        }
    }
}

fn rust_to_c_string(text: &str) -> CString {
    CString::new(text).unwrap()
}

pub struct Material {
    inner: *mut ffi::R3D_Material,
    weak: bool,
}

impl Material {
    pub fn get_albedo(&self) -> MapAlbedo {
        unsafe {
            MapAlbedo {
                inner: &mut (*self.inner).albedo,
            }
        }
    }

    pub fn from_raw_weak(inner: *mut ffi::R3D_Material) -> Self {
        Self { inner, weak: true }
    }
}

pub struct MapAlbedo {
    inner: *mut ffi::R3D_Material_R3D_MapAlbedo,
}

impl MapAlbedo {
    pub fn get_texture(&self) -> WeakTexture2D {
        unsafe {
            let texture = (*self.inner).texture;
            let texture = std::mem::transmute(texture);
            Texture2D::from_raw(texture).make_weak()
        }
    }

    pub fn set_texture(&self, texture: &Texture2D) {
        unsafe {
            (*self.inner).texture.id = texture.id;
            (*self.inner).texture.width = texture.width;
            (*self.inner).texture.height = texture.height;
            (*self.inner).texture.mipmaps = texture.mipmaps;
            (*self.inner).texture.format = texture.format;
        }
    }
}
