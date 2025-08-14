#version 460 core

out vec4 FragColor;

in vec2 TexCoord;
in vec3 lightColor;
flat in uint voxel_id;

uniform sampler2DArray textures;

void main()
{
    vec3 ambient =  0.8f * lightColor;
    FragColor = texture(textures, vec3(TexCoord, voxel_id - 1)) * vec4(ambient, 1.0);
    if(TexCoord.x < 0.01 || TexCoord.x > 0.99 || TexCoord.y < 0.01 || TexCoord.y > 0.99) {
        FragColor = vec4(1.0, 1.0, 1.0, 1.0);
    }
}