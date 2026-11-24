#version 460 core
// unit quad corner, (0,0) to (1,1)
layout (location = 0) in vec2 aPos;

uniform mat4 projection;
// x, y (bottom-left corner), width, height in pixels
uniform vec4 rect;

void main()
{
    gl_Position = projection * vec4(rect.xy + aPos * rect.zw, 0.0, 1.0);
}
