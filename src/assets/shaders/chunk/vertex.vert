#version 460 core
layout(location = 0) in int packOne;
layout(location = 1) in int packTwo;

int x,y,z;
flat out int ao;

uniform mat4 uniform_projection_view;
uniform mat4 uniform_model;

out vec2 TexCoord;
out vec3 fragPos;
out float shading;
flat out vec3 normalVector;
flat out uint voxel_id;
flat out int faceIndex;
flat out float material_ambient;
flat out float material_diffuse;
flat out float material_specular;

float unpack_bits_to_float(uint value, float min, float max, uint bits) {
    float normalized = (value & bits) / float(bits);
    return min + normalized * (max - min);
}

// Utility to reassemble uvec2 into uint64_t
void unpack64(uint d1, uint d2) {
    voxel_id = (d1 >> 14u) & 0x3FFFFu;
    x = int((d1 >> 7u) & 0x7Fu);
    y = int(d1 & 0x7Fu);
    z = int((d2 >> 25u) & 0x7Fu);
    faceIndex = int((d2 >> 22u) & 0x7u);
    material_ambient  = unpack_bits_to_float(d2 >> 16, 0.0, 1.0, 0x3Fu);
    material_diffuse  = unpack_bits_to_float(d2 >> 10, 0.0, 1.0, 0x3Fu);
    material_specular = unpack_bits_to_float(d2 >> 4, 0.0, 1.0, 0x3Fu);
    ao = int((d2 >> 2) & 0x2u);
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

const float face_shading[6] = float[6](
    1.0, 0.5,  // top bottom
    0.5, 0.8,  // right left
    0.5, 0.8   // front back
);

const float ao_values[4] = float[4](0.1, 0.25, 0.5, 1.0);

void main() {
    unpack64(packOne,packTwo);

    normalVector = normal[faceIndex];

    vec4 pos = uniform_model * vec4(x, y, z, 1.0f);
    fragPos = vec3(pos);

    shading = face_shading[faceIndex] * ao_values[ao];

    gl_Position = uniform_projection_view * pos;
}