use super::cmd::ModCmd;
use super::cmd::SceneHook;
use crate::errors;
use crate::sync::ChannelPair;
use crossbeam::channel;
use rlua::Lua;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

pub struct ScriptRunner {
    pub(crate) lua: Lua,
    pub(crate) chan: ChannelPair<Vec<ModCmd>>,
    pub(crate) init_script: PathBuf,
    pub(crate) lib_name: String,
    pub(crate) errors: channel::Sender<errors::Error>,
}

impl ScriptRunner {
    #[allow(dead_code)]
    pub fn spawn() -> ScriptRunner {
        unimplemented!()
    }

    pub fn run(&mut self) {
        let mut running = true;

        while running {
            // Receive command buffer from hub.
            if let Ok(cmds) = self.chan.receive() {
                running = self.on_receive(&cmds);

                // Send back to hub
                self.chan.send(cmds).unwrap();
            }
        }
    }

    /// Dispatch the incoming command to a handler method.
    ///
    /// Returns `true` to indicate that the receiver loop
    /// should continue, `false` if it should stop.
    fn on_receive(&mut self, cmds: &[ModCmd]) -> bool {
        use ModCmd::*;

        for cmd in cmds.iter() {
            match cmd {
                Init => {
                    self.handle(self.run_init());
                }
                Shutdown => {
                    return false;
                }
                Scene(hook) => self.handle(self.run_scene_hook(hook)),
                Game => unimplemented!(),
            }
        }

        true
    }

    /// Helper handler that will send errors over the runner's error channel.
    #[allow(unused_must_use)]
    fn handle<T>(&mut self, result: errors::Result<T>) {
        match result {
            Ok(_) => {}
            Err(err) => {
                self.errors.send(err);
            }
        }
    }

    fn run_init(&self) -> errors::Result<()> {
        let mut file = File::open(&self.init_script)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let result: rlua::Result<()> = self.lua.context(move |lua_ctx| {
            lua_ctx.load(&content).exec()?;

            Ok(())
        });

        result?;

        Ok(())
    }

    fn run_scene_hook(&self, hook: &SceneHook) -> errors::Result<()> {
        use super::cmd::SceneHook::*;

        let lib_name: &str = &self.lib_name;

        match hook {
            Start => {
                let result: rlua::Result<()> = self.lua.context(move |lua_ctx| {
                    let globals = lua_ctx.globals();
                    let lib_table: rlua::Table = globals.get(lib_name)?;

                    // The mod does not have to declare a hook.
                    let func_result: rlua::Result<rlua::Function> = lib_table.get("on_start");
                    if let Ok(func) = func_result {
                        func.call::<_, ()>(())?;
                    }

                    Ok(())
                });

                result?;

                Ok(())
            }
            Stop => {
                let result: rlua::Result<()> = self.lua.context(move |lua_ctx| {
                    let globals = lua_ctx.globals();
                    let lib_table: rlua::Table = globals.get(lib_name)?;

                    // The mod does not have to declare a hook.
                    let func_result: rlua::Result<rlua::Function> = lib_table.get("on_stop");
                    if let Ok(func) = func_result {
                        func.call::<_, ()>(())?;
                    }

                    Ok(())
                });

                result?;

                Ok(())
            }
        }
    }
}
