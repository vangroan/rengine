// shaders/basic_150.glslv
#version 150 core

in vec3 a_Pos;
in vec2 a_Uv;
in vec3 a_Normal;
in vec4 a_Color;
out vec2 v_Uv;
out vec4 v_Color;

uniform Transform {
    mat4 u_Transform;
};
uniform mat4 u_View;
uniform mat4 u_Proj;

void main() {
    v_Color = vec4(a_Color);
    v_Uv = a_Uv;
    gl_Position = u_Proj * u_View * u_Transform * vec4(a_Pos, 1.0);
}