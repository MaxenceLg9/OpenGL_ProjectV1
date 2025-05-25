#version 460 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aColor;
layout (location = 2) in vec2 aTexCoord;

uniform mat4 p_v_m;
uniform vec3 color;

out vec3 ourColor;
out vec2 TexCoord;
out vec3 lightColor;

void main()
{
    //gl_Position = vec4(aPos, 1.0);
    lightColor = color;
    TexCoord = aTexCoord;
    gl_Position = p_v_m * vec4(aPos, 1.0f);
}