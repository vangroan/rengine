// shaders/basic_150.glslf
#version 150 core

uniform sampler2D t_Sampler;

in vec2 v_Uv;
in vec4 v_Color;
out vec4 Target0;

void main() {
    vec4 aw = texture(t_Sampler, v_Uv).rgba;

    Target0 = aw * v_Color;
}