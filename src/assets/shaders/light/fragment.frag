#version 460 core

out vec4 FragColor;

in vec3 lightColor;

uniform sampler2D texture1;
uniform sampler2D texture2;
uniform float mixValue;

void main()
{
    FragColor = vec4(lightColor,1.0f);
}
