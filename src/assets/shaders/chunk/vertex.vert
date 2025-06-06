#version 460 core
layout(location = 0) in int packOne;
layout(location = 1) in int packTwo;

int x,y,z;
float nX, nY, nZ;

uniform mat4 p_v_m;
uniform vec3 color;

out vec2 TexCoord;
out vec3 lightColor;
out vec3 blockColor;
flat out uint voxel_id;

// Utility to reassemble uvec2 into uint64_t
void unpack64(int d1, int d2) {
    voxel_id = (d1 >> 14u) & 0x3FFFFu;
    x = int((d1 >> 7u) & 0x7Fu);
    y = int(d1 & 0x7Fu);
    z = int((d2 >> 25u) & 0x7Fu);
    nX = ((d2 >> 18u) & 0xFFu) / 511.5f - 1.0f;
    nY = ((d2 >> 10u) & 0xFFu) / 511.5f - 1.0f;
    nZ = ((d2 >> 2u) & 0xFFu) / 511.5f - 1.0f;
    TexCoord.x = (d2 >> 1u) & 0x1u;
    TexCoord.y = d2 & 0x1u;
}

void main() {
    unpack64(packOne,packTwo);
    uint debug = packTwo; // force the driver to keep it
    lightColor = color;
    blockColor = vec3(sin(x), cos(y), sin(z));
    gl_Position = p_v_m * vec4(x, y, z, 1.0f);
}