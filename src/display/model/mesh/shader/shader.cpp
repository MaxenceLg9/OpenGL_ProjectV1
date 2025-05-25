//
// Created by maxence on 02/05/25.
//

#include "shader.h"
#include <glad/glad.h>
#include <cstdio>
#include <cstdlib>

Shader::Shader(const char *vertexPath, const char *fragmentPath) {

    char *vertexCode = NULL, *fragmentCode = NULL;
    Shader::readFile(&vertexCode, vertexPath);
    Shader::readFile(&fragmentCode, fragmentPath);

    int success;
    char infoLog[512];

    // vertex Shader
    unsigned int vertex;
    unsigned int fragment;
    Shader::compileShader(&vertex, (const char **) &vertexCode,GL_VERTEX_SHADER,"GL_VERTEX_SHADER");
    Shader::compileShader(&fragment, (const char **) &fragmentCode,GL_FRAGMENT_SHADER,"GL_FRAGMENT_SHADER");

    // shader Program
    this->id = glCreateProgram();
    glAttachShader(this->id, vertex);
    glAttachShader(this->id, fragment);
    glLinkProgram(this->id);
    glGetProgramiv(this->id, GL_LINK_STATUS, &success);//debug
    if(!success) {
        glGetProgramInfoLog(this->id, 512, NULL, infoLog);
        printf("ERROR : Shader linking Failed %s\n",infoLog);
    }

    // delete the shaders as they're linked into our program now and no longer necessary
    glDeleteShader(vertex);
    glDeleteShader(fragment);
}

Shader::~Shader() {
    glDeleteProgram(this->id);
}

void Shader::readFile(char **buffer, const char *filename) {
    FILE *file = fopen(filename, "r");
    if (!file) {
        fprintf(stderr, "Could not open file %s\n", filename);
        return;
    }
    fseek(file, 0, SEEK_END);
    const long size = ftell(file);
    rewind(file);
    *buffer = (char *)(calloc(1, size + 1));
    if(*buffer == NULL ){
        printf("Could not allocate memory for file %s\n", filename);
        fclose(file);
        return;
    }
    fread(*buffer, 1, size, file);
    (*buffer)[size] = '\0';
    // printf("File %s loaded successfully\n", *buffer);
    fclose(file);
}

void Shader::compileShader(unsigned int *shader, const char **code, const int type, const char *typeName) {
    int success;

    *shader = glCreateShader(type);
    glShaderSource(*shader, 1, code, NULL);
    glCompileShader(*shader);
    glGetShaderiv(*shader, GL_COMPILE_STATUS, &success);//debug
    if(!success) {
        char infoLog[2048];
        glGetShaderInfoLog(*shader, 2048, NULL, infoLog);
        printf("ERROR : Shader %s Failed %s\n",typeName,infoLog);
    }
}

// use/activate the shader
void Shader::use() const {
    glUseProgram(this->id);
}

void Shader::setInt(const char *name, const int value) const {
    glUniform1i(glGetUniformLocation(this->id, name), value);
}

void Shader::setFloat(const char *name, const float value) const {
    glUniform1f(glGetUniformLocation(this->id, name), value);
}

void Shader::setVec2(const char *name, const float v1,const float v2) const {
    glUniform2f(glGetUniformLocation(this->id, name),v1, v2);
}

void Shader::setVec3(const char *name, float v1, float v2,float v3) const {
    glUniform3f(glGetUniformLocation(this->id, name),v1, v2,v3);
}

void Shader::setVec4(const char *name, const float v1,const float v2, const float v3, const float v4) const {
    glUniform4f(glGetUniformLocation(this->id, name),v1, v2, v3, v4);
}

void Shader::setMatrix4fv(const char *name, const float *matrix) const {
    glUniformMatrix4fv(glGetUniformLocation(this->id, name), 1, GL_FALSE, matrix);
}
