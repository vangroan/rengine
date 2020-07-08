#version 150 core

in vec3 a_Pos;
in vec2 a_Uv;
in vec3 a_Normal;
in vec4 a_Color;
out vec2 v_Uv;
out vec4 v_Color;
out vec3 v_FragPos;
out vec3 v_Normal;

uniform mat4 u_NormalMatrix;
uniform mat4 u_Model;
uniform mat4 u_View;
uniform mat4 u_Proj;

void main() {
    v_Color = vec4(a_Color);
    v_Uv = a_Uv;

    gl_Position = u_Proj * u_View * u_Model * vec4(a_Pos, 1.0);

    // Lighting is in world space, so we need the vertex's world space position
    // without the projection and view transforms applied.
    v_FragPos = vec3(u_Model * vec4(a_Pos, 1.0));
    
    // Normal matrix is casted to mat3 so it loses its translation components
    // and can be multiplied with a vec3.
    v_Normal = mat3(u_NormalMatrix) * a_Normal;
}