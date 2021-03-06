#![allow(clippy::single_component_path_imports)]
use gfx;

pub type ColorFormat = gfx::format::Srgba8;
pub type DepthFormat = gfx::format::DepthStencil;

pub type GraphicsEncoder = gfx::Encoder<gfx_device::Resources, gfx_device::CommandBuffer>;
pub type RenderTarget<R> = gfx::handle::RenderTargetView<R, ColorFormat>;
pub type DepthTarget<R> = gfx::handle::DepthStencilView<R, DepthFormat>;

// NOTE: that document comments inside this block breaks the macro
#[rustfmt::skip]
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

    // Note: Never use vec3 inside uniform block
    constant GlossMaterial {
        ambient: [f32; 4] = "u_Ambient",
        diffuse: [f32; 4] = "u_Diffuse",
        specular: [f32; 4] = "u_Specular",
        shininess: f32 = "u_Shininess",
    }

    // Note: Never use vec3 inside uniform block
    constant LightParams {
        pos: [f32; 4] = "pos",
        ambient: [f32; 4] = "ambient",
        diffuse: [f32; 4] = "diffuse",
        specular: [f32; 4] = "specular",
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
        render_target: gfx::BlendTarget<ColorFormat> = ("Target0", gfx::state::ColorMask::all(), gfx::preset::blend::ALPHA),

        depth_target: gfx::DepthTarget<DepthFormat> =
            gfx::preset::depth::LESS_EQUAL_WRITE,
    }

    pipeline gloss_pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),

        // Simple texture sampler
        sampler: gfx::TextureSampler<[f32; 4]> = "t_Sampler",

        // Phong material parameters
        material: gfx::ConstantBuffer<GlossMaterial> = "b_Material",

        // Support one point light
        lights: gfx::ConstantBuffer<LightParams> = "b_Lights",

        // Number of lights uploaded to buffer
        num_lights: gfx::Global<i32> = "u_NumLights",

        // Camera position
        eye: gfx::Global<[f32; 4]> = "u_Eye",

        normal_matrix: gfx::Global<[[f32; 4]; 4]> = "u_NormalMatrix",

        // Model Transform Matrix
        model: gfx::Global<[[f32; 4]; 4]> = "u_Model",

        // View
        view: gfx::Global<[[f32; 4]; 4]> = "u_View",

        // Projection
        proj: gfx::Global<[[f32; 4]; 4]> = "u_Proj",

        // Enables the scissor test
        scissor: gfx::Scissor = (),

        // out: gfx::RenderTarget<ColorFormat> = "Target0"
        // This makes the BlendMode part of the pipeline, which is fine for the simple case
        render_target: gfx::BlendTarget<ColorFormat> = ("Target0", gfx::state::ColorMask::all(), gfx::preset::blend::ALPHA),

        depth_target: gfx::DepthTarget<DepthFormat> =
            gfx::preset::depth::LESS_EQUAL_WRITE,
    }

    pipeline gizmo_pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),

        // Model Transform Matrix
        model: gfx::Global<[[f32; 4]; 4]> = "u_Model",

        // View
        view: gfx::Global<[[f32; 4]; 4]> = "u_View",

        // Projection
        proj: gfx::Global<[[f32; 4]; 4]> = "u_Proj",

        // Enables the scissor test
        scissor: gfx::Scissor = (),

        render_target: gfx::RenderTarget<ColorFormat> = "Target0",

        depth_target: gfx::DepthTarget<DepthFormat> = gfx::preset::depth::LESS_EQUAL_WRITE,
    }

    pipeline line_pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),

        // Model Transform Matrix
        model: gfx::Global<[[f32; 4]; 4]> = "u_Model",

        // View
        view: gfx::Global<[[f32; 4]; 4]> = "u_View",

        // Projection
        proj: gfx::Global<[[f32; 4]; 4]> = "u_Proj",

        // Enables the scissor test
        scissor: gfx::Scissor = (),

        // This makes the BlendMode part of the pipeline, which is fine for the simple case
        out: gfx::RenderTarget<ColorFormat> = "Target0",
    }

    pipeline gui_pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),

        // Simple texture sampler
        sampler: gfx::TextureSampler<[f32; 4]> = "t_Sampler",

        // Model Transform Matrix
        model: gfx::Global<[[f32; 4]; 4]> = "u_Model",

        // Projection
        proj: gfx::Global<[[f32; 4]; 4]> = "u_Proj",

        // Enables the scissor test
        scissor: gfx::Scissor = (),

        // This makes the BlendMode part of the pipeline, which is fine for the simple case
        render_target: gfx::BlendTarget<ColorFormat> = ("Target0", gfx::state::ColorMask::all(), gfx::preset::blend::ALPHA),

        depth_target: gfx::DepthTarget<DepthFormat> =
            gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}

pub type PipelineStateObject = gfx::PipelineState<gfx_device::Resources, pipe::Meta>;
pub type GizmoPso = gfx::PipelineState<gfx_device::Resources, gizmo_pipe::Meta>;
pub type LinePso = gfx::PipelineState<gfx_device::Resources, line_pipe::Meta>;
pub type ShaderProgram = gfx::handle::Program<gfx_device::Resources>;

#[allow(dead_code)]
pub struct PipelineBundle<M> {
    pub(crate) pso: gfx::PipelineState<gfx_device::Resources, M>,
    pub(crate) program: ShaderProgram,
}

impl<M> PipelineBundle<M> {
    pub fn new(pso: gfx::PipelineState<gfx_device::Resources, M>, program: ShaderProgram) -> Self {
        PipelineBundle { pso, program }
    }
}
