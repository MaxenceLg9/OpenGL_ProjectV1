#version 460 core

out vec4 FragColor;

in vec2 TexCoord;
in vec3 lightColor;
in vec3 blockColor;
flat in uint voxel_id;

uniform sampler2D texture1;

void main()
{
    float ambientStrength = 0.1;
    vec3 ambient = ambientStrength * lightColor;
//    FragColor = texture(texture1, TexCoord) * vec4(ambient * lightColor,1.0);
    FragColor = vec4(1.0) * vec4(ambient * lightColor,1.0);

}