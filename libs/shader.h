#ifndef SHADER_H
#define SHADER_H

#include <glad/glad.h> // include glad to get all the required OpenGL headers

typedef struct Shader {
    unsigned int ID; // shader program ID
    char *vertexCode;
    char *fragmentCode;
} SHADER;


// constructor reads and builds the shader
void getShader(SHADER *shader,const char *vertexPath, const char *fragmentPath);

// use/activate the shader
void use(const SHADER *shader);

void setInt(const SHADER *shader, const char *name, int value);

void setFloat(const SHADER *shader, const char *name, float value);

void setVec2(const SHADER *shader, const char *name, float v1, float v2);

void setVec4(const SHADER *shader, const char *name, float v1, float v2, float v3, float v4);

void setMatrix4fv(const SHADER *shader, const char *name, const float *matrix);

#endif
