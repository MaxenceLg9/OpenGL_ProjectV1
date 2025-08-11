#version 460 core

out vec4 FragColor;

in vec2 TexCoord;
in vec3 lightColor;
in vec3 blockColor;
flat in uint voxel_id;

uniform sampler2D texture1;

void main()
{
    vec3 ambient =  0.8f * lightColor;
    FragColor = texture(texture1, TexCoord) * vec4(ambient,1.0);
//    FragColor = texture(texture1, TexCoord);
//    FragColor = vec4(1.0) * vec4(ambient * lightColor,1.0);

}