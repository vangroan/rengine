#![allow(clippy::large_enum_variant)]

use crate::gfx_types::GraphicsEncoder;
use crate::scene::SceneError;
use crossbeam::channel::{RecvError, SendError};
use glutin::CreationError;

error_chain! {
    // Names driven by convention.
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    foreign_links {
        Fmt(::std::fmt::Error);
        Io(::std::io::Error);
        SceneTransition(SceneError);
        EncoderRecv(RecvError);
        GlutinCreate(CreationError);

        // `error-chain` does not currently support polymorphism.
        GraphicsEncoderSend(SendError<GraphicsEncoder>);
        Lua(rlua::Error);
        Toml(toml::de::Error);
    }

    errors {
        WindowSize {
            description("failed to retrieve window size")
            display("failed to retrieve window size")
        }
        NoInitialScene {
            description("no initial scene configured")
            display("no initial scene configured")
        }
        ModLoad {
            description("failed to load mods")
            display("failed to load mods")
        }
        ModScriptThread {
            description("script runner thread panic")
            display("script runner thread panic")
        }
        ModDispatch {
            description("did not receive command buffer back from script runner")
            display("did not receive command buffer back from script runner")
        }
        ModComposite(v: Vec<Error>) {
            description("multiple script failures")
            display("multiple script failures: {} errors", v.len())
        }
    }
}
