#version 460 core

out vec4 FragColor;

in vec2 TexCoord;
in vec3 lightColor;

uniform sampler2D texture1;
uniform sampler2D texture2;

void main()
{
    FragColor = texture(texture2, TexCoord) * vec4(lightColor,1.0);
}
