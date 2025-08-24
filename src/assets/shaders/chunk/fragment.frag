#version 460 core

in vec2 TexCoord;

flat in vec3 lightColor;
flat in uint voxel_id;
flat in int faceIndex;
flat in vec3 diffuse;

uniform sampler2DArray textures;

out vec4 FragColor;

void main() {
    vec3 ambient =  0.2f * lightColor;
    FragColor = texture(textures, vec3(TexCoord, voxel_id - 1)) * vec4(ambient + diffuse, 1.0);
    if(faceIndex > 0 && faceIndex < 5) {
        FragColor *= 0.9f;
    } else if(faceIndex == 5) {
        FragColor *= 0.4f;
    }
}