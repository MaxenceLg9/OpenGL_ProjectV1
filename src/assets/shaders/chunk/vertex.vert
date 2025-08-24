#version 460 core
layout(location = 0) in int packOne;
layout(location = 1) in int packTwo;

int x,y,z;
vec3 normalVector;

uniform mat4 uniform_projection_view;
uniform mat4 uniform_model;
uniform vec3 uniformLightColor;
uniform vec3 uniformLightPos;

out vec2 TexCoord;
flat out vec3 lightColor;
flat out uint voxel_id;
flat out vec3 diffuse;
flat out int faceIndex;

// Utility to reassemble uvec2 into uint64_t
void unpack64(uint d1, uint d2) {
    voxel_id = (d1 >> 14u) & 0x3FFFFu;
    x = int((d1 >> 7u) & 0x7Fu);
    y = int(d1 & 0x7Fu);
    z = int((d2 >> 25u) & 0x7Fu);
    faceIndex = int((d2 >> 18u) & 0x7u);
    TexCoord.x = (d2 >> 1u) & 0x1u;
    TexCoord.y = d2 & 0x1u;
}

vec3 normal[] = {
    vec3( 0.0,  1.0,  0.0), // top
    vec3( 0.0,  0.0,  1.0), // front
    vec3( 1.0,  0.0,  0.0), // right
    vec3(-1.0,  0.0,  0.0), // left
    vec3( 0.0,  0.0, -1.0), // back
    vec3( 0.0, -1.0,  0.0)  // bottom
};

void main() {
    unpack64(packOne,packTwo);

    normalVector = normal[faceIndex];

    vec4 pos = uniform_model * vec4(x, y, z, 1.0f);
    vec3 fragPos = vec3(pos);
    vec3 lightDir = normalize(uniformLightPos - fragPos);
    float diff = max(dot(normalVector, lightDir), 0.0);

    diffuse = diff * uniformLightColor;
    lightColor = uniformLightColor;

    gl_Position = uniform_projection_view * pos;

}