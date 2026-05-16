#version 460 core
in vec2 TexCoord;
in vec3 fragPos;

flat in uint voxel_id;

uniform sampler2DArray characters;
uniform vec3 textColor;

out vec4 FragColor;

void main() {
    vec4 sampled = vec4(1.0, 1.0, 1.0, texture(characters, vec3(TexCoord, voxel_id)).r);
    FragColor = vec4(textColor, 1.0) * sampled;
    if (texture(characters, vec3(TexCoord, voxel_id)).r == 0) {
        FragColor = vec4(1.0,1.0,1.0,1.0);
    }

//    FragColor = vec4(1.0,1.0,1.0, 1.0) * vec4(textColor, 1.0);
}