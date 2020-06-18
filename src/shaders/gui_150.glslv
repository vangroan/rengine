// shaders/basic_150.glslv
#version 150 core

const mat4 INVERT_Y_AXIS = mat4(
    vec4(1.0, 0.0, 0.0, 0.0),
    vec4(0.0, -1.0, 0.0, 0.0),
    vec4(0.0, 0.0, 1.0, 0.0),
    vec4(0.0, 0.0, 0.0, 1.0)
);

in vec3 a_Pos;
in vec2 a_Uv;
in vec3 a_Normal;
in vec4 a_Color;
out vec2 v_Uv;
out vec4 v_Color;

uniform mat4 u_Model;
uniform mat4 u_Proj;

void main() {
    v_Color = vec4(a_Color);
    v_Uv = a_Uv;
    gl_Position = INVERT_Y_AXIS * u_Proj * u_Model * vec4(a_Pos, 1.0);
}