// shaders/gizmo_150.glslv
#version 150 core

in vec3 a_Pos;
in vec4 a_Color;
out vec4 v_Color;

uniform mat4 u_Model;
uniform mat4 u_View;
uniform mat4 u_Proj;

void main() {
    v_Color = vec4(a_Color);
    gl_Position = u_Proj * u_View * u_Model * vec4(a_Pos, 1.0);
}