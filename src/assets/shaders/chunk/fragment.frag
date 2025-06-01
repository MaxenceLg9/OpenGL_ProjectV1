#version 460 core

out vec4 FragColor;

in vec2 TexCoord;
in vec3 lightColor;
flat in uint voxel_id;

uniform sampler2D texture1;
uniform sampler2D texture2;

void main()
{
    float ambientStrength = 0.1;
    vec3 ambient = ambientStrength * lightColor;
//    FragColor = texture(texture2, TexCoord) * vec4(ambient * lightColor,1.0);
    FragColor = vec4(vec3(float(voxel_id) / 64.0), 1.0);

}