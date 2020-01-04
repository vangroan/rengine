// shaders/basic_150.glslf
#version 150 core

uniform sampler2D t_Sampler;

in vec2 v_Uv;
in vec4 v_Color;
out vec4 Target0;

void main() {
    vec4 texel = texture(t_Sampler, v_Uv).rgba;
    // Prevent transparent pixels from overwriting opaque pixels in the back.
    if (texel.a < 0.5) {
        discard;
    }
    Target0 = texel * v_Color;
}