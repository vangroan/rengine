use super::chan::ChannelPair;
use super::cmd::ModCmd;
use crate::errors;
use crate::intern::InternedStr;
use crossbeam::channel;
use rlua::Lua;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

pub struct ScriptRunner {
    pub(crate) lua: Lua,
    pub(crate) chan: ChannelPair,
    pub(crate) init_script: PathBuf,
    pub(crate) errors: channel::Sender<errors::Error>,
}

impl ScriptRunner {
    pub fn spawn() -> ScriptRunner {
        unimplemented!()
    }

    pub fn run(&mut self) {
        let mut running = true;

        'main: while running {
            // Receive command buffer from hub
            match self.chan.receive() {
                Ok(mut cmds) => {
                    running = self.on_receive(&mut cmds);

                    // Send back to hub
                    self.chan.send(cmds).unwrap();
                }
                _ => {}
            }
        }
    }

    /// Dispatch the incoming command to a handler method.
    ///
    /// Returns `true` to indicate that the receiver loop
    /// should continue, `false` if it should stop.
    fn on_receive(&mut self, cmds: &Vec<ModCmd>) -> bool {
        use ModCmd::*;

        for cmd in cmds.iter() {
            match cmd {
                Init => {
                    self.handle(self.run_init());
                }
                Shutdown => {
                    return false;
                }
            }
        }

        return true;
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
}
