#version 460 core
out vec4 FragColor;

in vec3 ourColor;
in vec2 TexCoord;


uniform vec4 color;
uniform sampler2D texture1;
uniform sampler2D texture2;
uniform float mixValue;

void main()
{
    //FragColor = mix(texture(texture1, vec2(1.0 - TexCoord.x,1.0- TexCoord.y)), texture(texture2, TexCoord), mixValue);
    FragColor = mix(texture(texture1, TexCoord), texture(texture2, TexCoord), mixValue);
    //FragColor = texture(texture1, TexCoord);
    //FragColor = color;
}
