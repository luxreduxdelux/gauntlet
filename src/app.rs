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

use crate::helper::*;
use crate::user::*;
use crate::view::*;
use crate::world::*;

//================================================================

use raylib::prelude::*;

//================================================================

/// The main app state.
#[derive(Default)]
pub struct App<'a> {
    /// Whether or not to close the app.
    pub close: bool,
    /// Active game world, if any.
    pub world: Option<World<'a>>,
    /// User interface.
    pub view: View<'a>,
    /// User configuration.
    pub user: User,
}

impl<'a> App<'a> {
    pub const VERSION: &'a str = env!("CARGO_PKG_VERSION");

    /// The app's main loop.
    pub fn main() -> anyhow::Result<()> {
        // Set panic hook and backtrace for debugging.
        unsafe {
            std::env::set_var("RUST_BACKTRACE", "1");

            std::panic::set_hook(Box::new(|panic_info| {
                let time = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap();
                let file = format!("panic_{time:?}");
                std::fs::write(&file, panic_info.to_string()).unwrap();
                error_message(&format!(
                    "Fatal error! The log \"{file}\" has been written to your game's root directory."
                ));

                eprintln!("{panic_info}");
            }));
        }

        //================================================================

        let mut context = Context::new()?;
        let mut app = Self::default();

        //================================================================

        context.apply_user(&app.user);

        unsafe {
            let context = &mut context as *mut Context;
            app.initialize(&mut *context)?;
        };

        //================================================================

        // Run loop for as long as window should be open and the user hasn't sent a close signal.
        while !context.handle.window_should_close() && !app.close {
            let ctx = { &mut context as *mut Context };

            unsafe {
                let mut draw = context.handle.begin_drawing(&context.thread);

                draw.clear_background(Color::BLACK);

                let app = &mut app as *mut Self;

                if let Some(world) = &mut (*app).world {
                    world.main(&mut *app, &mut draw, &mut *ctx)?;
                }

                View::draw_layout(&mut *app, &mut draw, &mut *ctx)?;
            }
        }

        Ok(())
    }

    /// Initialize the app proper after context is ready.
    pub fn initialize(&mut self, context: &mut Context) -> anyhow::Result<()> {
        let app = { self as *mut Self };
        let ctx = { context as *mut Context };

        self.view
            .initialize(unsafe { &mut *app }, unsafe { &mut *ctx })?;

        Ok(())
    }

    /// Initialize a new game world.
    pub fn new_world(&mut self, context: &mut Context) -> anyhow::Result<()> {
        Layout::set_layout(self, &mut context.handle, None);
        self.world = Some(World::new(self, context)?);

        Ok(())
    }
}

//================================================================

/// The RL context.
pub struct Context {
    /// RL handle.
    pub handle: RaylibHandle,
    /// RL thread.
    pub thread: RaylibThread,
    /// RL handle (audio).
    pub audio: RaylibAudio,
}

impl Context {
    /// Create a new context, that being a Raylib instance as well as a R3D instance.
    pub fn new() -> anyhow::Result<Self> {
        let (mut handle, thread) = raylib::init()
            .size(1024, 768)
            .resizable()
            .title("Gauntlet Complex")
            .build();

        handle.set_exit_key(None);
        //handle.set_trace_log(TraceLogLevel::LOG_ERROR);

        let audio = RaylibAudio::init_audio_device()?;

        Ok(Self {
            handle,
            thread,
            audio,
        })
    }

    /// Apply the user's configuration data to the context.
    pub fn apply_user(&mut self, user: &User) {
        if user.video_full {
            let i = get_current_monitor();
            self.handle
                .set_window_size(get_monitor_width(i), get_monitor_height(i));
            self.handle.toggle_fullscreen();
        }

        self.handle.set_target_fps(user.video_rate as u32);
    }
}
