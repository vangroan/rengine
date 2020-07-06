#version 150 core

// precision highp float;

const int MAX_LIGHTS = 1;

struct Material {
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
    float shininess;
};

struct Light {
    vec3 pos;
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};

in vec2 v_Uv;
in vec4 v_Color;
in vec3 v_FragPos;
in vec3 v_Normal;
out vec4 Target0;

layout(std140) uniform b_Material {
    // Material u_Material;
    vec3 u_Ambient;
    vec3 u_Diffuse;
    vec3 u_Specular;
    float u_Shininess;
};


uniform b_Light {
    Light u_Light[MAX_LIGHTS];
    // vec3 pos;
    // vec3 ambient;
    // vec3 diffuse;
    // vec3 specular;
};

void main() {
    // ambient
    vec3 a = u_Ambient;
    // vec3 ambient = material.ambient * light.u_Light.ambient;

    // // diffuse
    // vec3 norm = normalize(v_Normal);
    // vec3 lightDir = normalize(u_Light.pos - v_FragPos);
    // float diff = max(dot(v_Normal, lightDir), 0.0);
    // vec3 diffuse = u_Light.diffuse * (diff * u_Material.diffuse);

    // // vec3 result = ambient + diffuse + specular;
    // vec3 result = ambient + diffuse;
    // Target0 = vec4(result, 1.0);
}
