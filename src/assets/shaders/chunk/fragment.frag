#version 460 core

in vec2 TexCoord;
in vec3 fragPos;

const vec3 gamma = vec3(2.2);
const vec3 inv_gamma = 1 / gamma;

flat in uint voxel_id;
flat in int faceIndex;
flat in vec3 normalVector;
flat in float material_ambient;
flat in float material_diffuse;
flat in float material_specular;
flat in int ao;

in float shading;
in vec3 relativeFragPos;


uniform sampler2DArray textures;
uniform vec3 uniformBgColor;
uniform vec3 uniformLightPos;
uniform vec3 uniformLightColor;
uniform vec3 uniformViewPos;

out vec4 FragColor;
out float depth;

void main() {
    vec3 lightDir = normalize(uniformLightPos - fragPos);

    vec3 viewDir = normalize(uniformViewPos - fragPos);
    vec3 reflectDir = reflect(-lightDir, normalVector);
    float specular = pow(max(dot(viewDir, reflectDir), 0.0), 32) * material_specular;

    float diffuse = max(dot(normalVector, lightDir), 0.0) * material_diffuse;

    lightDir = normalize(uniformLightPos + vec3(1000.0f,0.0f,0.0f) - fragPos);
    diffuse += max(dot(normalVector, lightDir), 0.0) * material_diffuse;

    lightDir = normalize(uniformLightPos + vec3(0.0f,1000.0f,0.0f) - fragPos);
    diffuse += max(dot(normalVector, lightDir), 0.0) * material_diffuse;

    lightDir = normalize(uniformLightPos + vec3(0.0f,0.0f,1000.0f) - fragPos);
    diffuse += max(dot(normalVector, lightDir), 0.0) * material_diffuse;

    float distance = length(relativeFragPos);
    // float distance = min(distance, 1000.0);
    float near = 400.0;
    if ( distance < near) {
        depth = 0.0;
    } else {
        // depth = (1 - far / near) * distance + (far / near);
        depth = (1.0 - exp(-0.01 * pow(distance - near, 2.0)));
    }
    // depth = 1.0f / depth;

    // float fog_dist = 1.0 / gl_FragCoord.w;
    // float fog_dist = 1.0 / gl_FragDepth;

    vec4 tex_col = texture(textures, vec3(TexCoord, voxel_id));

    // tex_col.rgb = pow(tex_col.rgb, gamma) * vec3(shading);
    tex_col.rgb = tex_col.rgb * vec3(shading);
    // tex_col.rgb = pow(tex_col.rgb, inv_gamma);

    tex_col.rgb = mix(tex_col.rgb, uniformBgColor, depth);

    FragColor = tex_col;
}
