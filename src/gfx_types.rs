use crate::colors::WHITE;
use gfx;

pub type ColorFormat = gfx::format::Srgba8;
pub type DepthFormat = gfx::format::DepthStencil;

pub type GraphicsEncoder = gfx::Encoder<gfx_device::Resources, gfx_device::CommandBuffer>;
pub type RenderTarget<R> = gfx::handle::RenderTargetView<R, ColorFormat>;

gfx_defines! {
    vertex Vertex {
        pos: [f32; 3] = "a_Pos",
        color: [f32; 4] = "a_Color",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        
        // Model Transform Matrix
        model: gfx::Global<[[f32; 4]; 4]> = "u_Model",

        // Enables the scissor test
        scissor: gfx::Scissor = (),

        out: gfx::RenderTarget<ColorFormat> = "Target0",
    }
}

pub type PipelineStateObject = gfx::PipelineState<gfx_device::Resources, pipe::Meta>;

pub const QUAD_VERTICES: [Vertex; 4] = [
    Vertex {
        pos: [0.0, 0.0, 0.0],
        color: WHITE,
    },
    Vertex {
        pos: [1.0, 0.0, 0.0],
        color: WHITE,
    },
    Vertex {
        pos: [1.0, 1.0, 0.0],
        color: WHITE,
    },
    Vertex {
        pos: [0.0, 1.0, 0.0],
        color: WHITE,
    },
];

pub const QUAD_INDICES: [u16; 6] = [
    // triangle 1
    0, 1, 2, // triangle 2
    0, 2, 3,
];
