// shaders/basic_150.glslv
#version 150 core

in vec3 a_Pos;
in vec4 a_Color;
out vec4 v_Color;

uniform Transform {
    mat4 u_Transform;
};

void main() {
    v_Color = vec4(a_Color);
    gl_Position = u_Transform * vec4(a_Pos, 1.0);
}