#version 460 core
layout (location = 0) in vec3 aPos;

uniform mat4 p_v_m;
uniform vec3 color;

out vec3 lightColor;

void main()
{
    lightColor = color;
    gl_Position = p_v_m * vec4(aPos, 1.0f);
}