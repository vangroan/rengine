// shaders/gizmo_150.glslf
#version 150 core

in vec2 v_Uv;
in vec4 v_Color;
out vec4 Target0;

void main() {
    Target0 = v_Color;
}