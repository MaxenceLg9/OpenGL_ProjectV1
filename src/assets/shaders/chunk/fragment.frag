#version 460 core

in vec2 TexCoord;
in vec3 fragPos;

flat in uint voxel_id;
flat in int faceIndex;
flat in vec3 normalVector;
flat in float material_ambient;
flat in float material_diffuse;
flat in float material_specular;

uniform sampler2DArray textures;
uniform vec3 uniformLightPos;
uniform vec3 uniformLightColor;
uniform vec3 uniformViewPos;

out vec4 FragColor;

void main() {
    float ambient = 0.5f;

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

    FragColor = vec4((ambient + diffuse + specular) * uniformLightColor, 1.0) * texture(textures, vec3(TexCoord, voxel_id - 1));
    if (faceIndex > 0 && faceIndex < 5) {
        FragColor *= 0.9f;
    } else if (faceIndex == 5) {
        FragColor *= 0.9f;
    }
}