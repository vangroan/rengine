#version 150 core

// precision lowp float;

const int MAX_LIGHTS = 1;

struct Light {
    vec4 pos;
    vec4 ambient;
    vec4 diffuse;
    vec4 specular;
};

in vec2 v_Uv;
in vec4 v_Color;
in vec3 v_FragPos;
in vec3 v_Normal;
out vec4 Target0;

layout(std140)
uniform b_Material {
    vec4 u_Ambient;
    vec4 u_Diffuse;
    vec4 u_Specular;
    float u_Shininess;
};

layout(std140)
uniform b_Lights {
    Light u_Lights[MAX_LIGHTS];
};

uniform vec4 u_Eye;
uniform sampler2D t_Sampler;

void main() {
    // ambient
    // vec3 a = u_Ambient;
    // vec3 ambient = material.ambient * light.u_Light.ambient;

    // // diffuse
    // vec3 norm = normalize(v_Normal);
    // vec3 lightDir = normalize(u_Light.pos - v_FragPos);
    // float diff = max(dot(v_Normal, lightDir), 0.0);
    // vec3 diffuse = u_Light.diffuse * (diff * u_Material.diffuse);

    // // vec3 result = ambient + diffuse + specular;
    // vec3 result = ambient + diffuse;
    // Target0 = vec4(result, 1.0);

    vec4 texel = texture(t_Sampler, v_Uv).rgba;

    // Prevent transparent pixels from overwriting opaque pixels in the back.
    if (texel.a < 0.5) {
        discard;
    }

    // vec4 color = vec4(1.0, 1.0, 1.0, 1.0);
    for (int i=0; i<MAX_LIGHTS; ++i) {
        Light light = u_Lights[i];
        
        // ambient
        vec4 ambient = u_Ambient * light.ambient;

        // diffuse
        vec3 norm = normalize(v_Normal);
        vec3 lightDir = normalize(vec3(light.pos) - v_FragPos);
        float diff = max(dot(norm, lightDir), 0.0);
        vec4 diffuse = light.diffuse * (diff * u_Diffuse);

        // specular
        vec3 viewDir = normalize(vec3(u_Eye) - v_FragPos);
        vec3 reflectDir = reflect(-lightDir, norm);  
        float spec = pow(max(dot(viewDir, reflectDir), 0.0), u_Shininess);
        vec4 specular = light.specular * (spec * u_Specular); 
        
        texel = texel * (ambient + diffuse + specular);
    }

    Target0 = texel * v_Color;
}
