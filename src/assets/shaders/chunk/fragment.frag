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
in float shading;
flat in int ao;

uniform sampler2DArray textures;
uniform vec3 uniformBgColor;
uniform vec3 uniformLightPos;
uniform vec3 uniformLightColor;
uniform vec3 uniformViewPos;

out vec4 FragColor;

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

    float fog_dist = 1.0 / gl_FragCoord.w;
    // float fog_dist = 1.0 / gl_FragDepth;
    vec4 tex_col = texture(textures, vec3(TexCoord, voxel_id));

    tex_col.rgb = pow(tex_col.rgb, gamma) * vec3(shading);
    tex_col.rgb = tex_col.rgb * vec3(shading);
    tex_col.rgb = pow(tex_col.rgb, inv_gamma);

    tex_col.rgb = mix(tex_col.rgb, uniformBgColor, (1.0 - exp2(-0.00006 * fog_dist * fog_dist)));

    FragColor = tex_col;
}
