#version 460 core
layout (location = 0) in vec3 aPos;

uniform mat4 view;
uniform mat4 model;
uniform mat4 projection;
uniform vec3 color;

out vec3 lightColor;

void main()
{
    gl_Position = vec4(aPos, 1.0);
    lightColor = color;
    gl_Position = projection * view * model * vec4(aPos, 1.0f);
}