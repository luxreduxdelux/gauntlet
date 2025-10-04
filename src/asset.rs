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

use crate::app::*;

//================================================================

use hashbrown::HashMap;
use raylib::prelude::*;
use serde::Deserialize;

//================================================================

/// Asset resource manager.
#[derive(Default)]
pub struct Asset<'a> {
    /// Model hash-map.
    model: HashMap<String, AssetModel>,
    /// Texture hash-map.
    texture: HashMap<String, Texture2D>,
    /// Shader hash-map.
    shader: HashMap<String, Shader>,
    /// Sound hash-map.
    sound: HashMap<String, AssetSound<'a>>,
    /// Music hash-map.
    music: HashMap<String, Music<'a>>,
    /// Font hash-map.
    font: HashMap<String, Font>,
}

impl<'a> Asset<'a> {
    const ANIMATION_BASE_FRAME_RATE: i32 = 60;

    /// Create a new model.
    pub fn set_model(
        &mut self,
        context: &mut Context,
        name: &str,
    ) -> anyhow::Result<&mut AssetModel> {
        if self.has_model(name) {
            return self.get_model(name);
        }

        let mut model = context.handle.load_model(&context.thread, name)?;
        let animation = context
            .handle
            .load_model_animations(&context.thread, name)
            .unwrap_or_default();
        let meta = ModelMeta::new(name)?;

        if let Some(texture) = &meta.texture {
            let material = model.materials_mut();

            for (i, path) in texture.iter().enumerate() {
                let texture = self.set_texture(context, &format!("data/{path}"))?;

                println!("set {i} texture");
                material[i + 1].maps_mut()[MaterialMapIndex::MATERIAL_MAP_ALBEDO as usize]
                    .texture = **texture;
                //material[i].get_albedo().set_texture(texture);
            }
        }

        self.model.insert(
            name.to_string(),
            AssetModel {
                model,
                animation,
                event: ModelEvent::from_model_meta(meta),
            },
        );

        self.get_model(name)
    }

    /// Get a model.
    pub fn get_model(&mut self, name: &str) -> anyhow::Result<&mut AssetModel> {
        self.model.get_mut(name).ok_or(anyhow::Error::msg(format!(
            "Asset::get_model(): Could not find asset \"{name}\"."
        )))
    }

    /// Check if the resource map has a model.
    pub fn has_model(&self, name: &str) -> bool {
        self.model.contains_key(name)
    }

    //================================================================

    /// Create a new texture.
    pub fn set_texture(
        &mut self,
        context: &mut Context,
        name: &str,
    ) -> anyhow::Result<&mut Texture2D> {
        if self.has_texture(name) {
            return self.get_texture(name);
        }

        let texture = context.handle.load_texture(&context.thread, name)?;

        self.texture.insert(name.to_string(), texture);

        self.get_texture(name)
    }

    /// Get a texture.
    pub fn get_texture(&mut self, name: &str) -> anyhow::Result<&mut Texture2D> {
        self.texture.get_mut(name).ok_or(anyhow::Error::msg(format!(
            "Asset::get_texture(): Could not find asset \"{name}\"."
        )))
    }

    /// Check if the resource map has a texture.
    pub fn has_texture(&self, name: &str) -> bool {
        self.texture.contains_key(name)
    }

    //================================================================

    /// Create a shader.
    pub fn set_shader(
        &mut self,
        context: &mut Context,
        name: &str,
        path_vs: Option<&str>,
        path_fs: Option<&str>,
    ) -> anyhow::Result<&mut Shader> {
        if self.has_shader(name) {
            return self.get_shader(name);
        }

        let shader = context
            .handle
            .load_shader(&context.thread, path_vs, path_fs);

        self.shader.insert(name.to_string(), shader);

        self.get_shader(name)
    }

    /// Get a shader.
    pub fn get_shader(&mut self, name: &str) -> anyhow::Result<&mut Shader> {
        self.shader.get_mut(name).ok_or(anyhow::Error::msg(format!(
            "Asset::get_shader(): Could not find asset \"{name}\"."
        )))
    }

    /// Check if the resource map has a shader.
    pub fn has_shader(&self, name: &str) -> bool {
        self.shader.contains_key(name)
    }

    //================================================================

    /// Create a sound handle.
    pub fn set_sound(
        &mut self,
        context: &'a Context,
        name: &str,
        alias_count: usize,
    ) -> anyhow::Result<&AssetSound<'a>> {
        if self.has_sound(name) {
            return self.get_sound(name);
        }

        let sound = context.audio.new_sound(name)?;

        let mut alias = Vec::with_capacity(alias_count);

        unsafe {
            let sound = &sound as *const Sound;

            for _ in 0..alias_count {
                alias.push((*sound).alias()?);
            }
        }

        self.sound
            .insert(name.to_string(), AssetSound { sound, alias });

        self.get_sound(name)
    }

    /// Get a sound handle.
    pub fn get_sound(&self, name: &str) -> anyhow::Result<&AssetSound<'a>> {
        self.sound.get(name).ok_or(anyhow::Error::msg(format!(
            "Asset::get_sound(): Could not find asset \"{name}\"."
        )))
    }

    /// Check if the resource map has a sound.
    pub fn has_sound(&self, name: &str) -> bool {
        self.sound.contains_key(name)
    }

    //================================================================

    /// Create a music handle.
    pub fn set_music(&mut self, context: &'a Context, name: &str) -> anyhow::Result<&Music<'a>> {
        if self.has_music(name) {
            return self.get_music(name);
        }

        let music = context.audio.new_music(name)?;

        self.music.insert(name.to_string(), music);

        self.get_music(name)
    }

    /// Get a music handle.
    pub fn get_music(&self, name: &str) -> anyhow::Result<&Music<'a>> {
        self.music.get(name).ok_or(anyhow::Error::msg(format!(
            "Asset::get_music(): Could not find asset \"{name}\"."
        )))
    }

    /// Check if the resource map has a music.
    pub fn has_music(&self, name: &str) -> bool {
        self.music.contains_key(name)
    }

    //================================================================

    /// Create a font.
    pub fn set_font(
        &mut self,
        context: &mut Context,
        name: &str,
        size: i32,
    ) -> anyhow::Result<&Font> {
        if self.has_font(name) {
            return self.get_font(name);
        }

        let font = context
            .handle
            .load_font_ex(&context.thread, name, size, None)?;

        self.font.insert(name.to_string(), font);

        self.get_font(name)
    }

    /// Get a font.
    pub fn get_font(&self, name: &str) -> anyhow::Result<&Font> {
        self.font.get(name).ok_or(anyhow::Error::msg(format!(
            "Asset::get_font(): Could not find asset \"{name}\"."
        )))
    }

    /// Check if the resource map has a font.
    pub fn has_font(&self, name: &str) -> bool {
        self.font.contains_key(name)
    }
}

impl Drop for Asset<'_> {
    fn drop(&mut self) {
        // TO-DO manually un-load each texture for each model, as raylib does not normally do that.
    }
}

//================================================================

/// A 3D model.
pub struct AssetModel {
    /// Handle to the R3D model data.
    pub model: Model,
    /// Handle to the R3D model animation data.
    pub animation: Vec<ModelAnimation>,
    /// Model-specific animation event data.
    pub event: ModelEvent,
}

/// Model animation event data.
#[derive(Debug, Deserialize, Default)]
pub struct ModelEvent {
    pub map: HashMap<String, HashMap<i32, AnimationEvent>>,
}

impl ModelEvent {
    fn from_model_meta(meta: ModelMeta) -> Self {
        if let Some(animation) = meta.event {
            Self { map: animation }
        } else {
            Self::default()
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum AnimationEvent {
    Sound { path: String },
    Custom(HashMap<String, serde_json::Value>),
}

#[derive(Debug, Deserialize, Default)]
struct ModelMeta {
    texture: Option<Vec<String>>,
    event: Option<HashMap<String, HashMap<i32, AnimationEvent>>>,
}

impl ModelMeta {
    const FILE_EXTENSION: &str = "meta";

    fn new(path: &str) -> anyhow::Result<Self> {
        let path: Vec<&str> = path.split(".").collect();
        let path = format!("{}.{}", path[0], Self::FILE_EXTENSION);

        if std::fs::exists(&path)? {
            Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
        } else {
            Ok(Self::default())
        }
    }
}

//================================================================

/// A sound.
pub struct AssetSound<'a> {
    /// Handle to the sound data.
    pub sound: Sound<'a>,
    /// Sound alias list, for playing back more than one sound at a time.
    pub alias: Vec<SoundAlias<'a, 'a>>,
}
