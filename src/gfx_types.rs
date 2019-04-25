use gfx;

pub type ColorFormat = gfx::format::Srgba8;
pub type DepthFormat = gfx::format::DepthStencil;

pub type GraphicsEncoder = gfx::Encoder<gfx_device::Resources, gfx_device::CommandBuffer>;
pub type RenderTarget<R> = gfx::handle::RenderTargetView<R, ColorFormat>;

/// Note that document comments inside this block breaks the macro
gfx_defines! {
    vertex Vertex {
        pos: [f32; 3] = "a_Pos",
        uv: [f32; 2] = "a_Uv",
        normal: [f32; 3] = "a_Normal",
        color: [f32; 4] = "a_Color",
    }

    constant Transform {
        transform: [[f32; 4]; 4] = "u_Transform",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),

        // Simple texture sampler
        sampler: gfx::TextureSampler<[f32; 4]> = "t_Sampler",

        // Model Transform Matrix
        transforms: gfx::ConstantBuffer<Transform> = "Transform",

        // View
        view: gfx::Global<[[f32; 4]; 4]> = "u_View",

        // Projection
        proj: gfx::Global<[[f32; 4]; 4]> = "u_Proj",

        // Enables the scissor test
        scissor: gfx::Scissor = (),

        // out: gfx::RenderTarget<ColorFormat> = "Target0"
        // This makes the BlendMode part of the pipeline, which is fine for the simple case
        out: gfx::BlendTarget<ColorFormat> = ("Target0", gfx::state::ColorMask::all(), gfx::preset::blend::ALPHA),
    }
}

pub type PipelineStateObject = gfx::PipelineState<gfx_device::Resources, pipe::Meta>;
pub type ShaderProgram = gfx::handle::Program<gfx_device::Resources>;

#[allow(dead_code)]
pub struct PipelineBundle {
    pub(crate) pso: PipelineStateObject,
    pub(crate) program: ShaderProgram,
}

impl PipelineBundle {
    pub fn new(pso: PipelineStateObject, program: ShaderProgram) -> Self {
        PipelineBundle { pso, program }
    }
}
