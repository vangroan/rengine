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
    }
}
