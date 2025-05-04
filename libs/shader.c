//
// Created by maxence on 02/05/25.
//

#include "shader.h"
#include <stdio.h>
#include <stdlib.h>

void readFile(char **buffer, const char *filename) {
    FILE *file = fopen(filename, "r");
    if (!file) {
        fprintf(stderr, "Could not open file %s\n", filename);
        return;
    }
    fseek(file, 0, SEEK_END);
    const long size = ftell(file);
    rewind(file);
    *buffer = malloc(size + 1);
    fread(*buffer, 1, size, file);
    (*buffer)[size] = '\0';
    fclose(file);
}

void compileShader(unsigned int *shader, const char **code, const int type) {
    int success;

    *shader = glCreateShader(type);
    glShaderSource(*shader, 1, code, NULL);
    glCompileShader(*shader);
    glGetShaderiv(*shader, GL_COMPILE_STATUS, &success);//debug
    if(!success) {
        char infoLog[512];
        glGetShaderInfoLog(*shader, 512, NULL, infoLog);
        printf("ERROR : Shader Fragment Compilation Failed %s\n",infoLog);
    }
}

void getShader(SHADER *shader,const char *vertexPath, const char *fragmentPath) {
    readFile(&shader->vertexCode, vertexPath);
    readFile(&shader->fragmentCode, fragmentPath);

    int success;
    char infoLog[512];

    // vertex Shader
    unsigned int vertex;
    unsigned int fragment;
    compileShader(&vertex, (const char **) &shader->vertexCode,GL_VERTEX_SHADER);
    compileShader(&fragment, (const char **) &shader->fragmentCode,GL_FRAGMENT_SHADER);

    // shader Program
    shader->ID = glCreateProgram();
    glAttachShader(shader->ID, vertex);
    glAttachShader(shader->ID, fragment);
    glLinkProgram(shader->ID);
    glGetProgramiv(shader->ID, GL_LINK_STATUS, &success);//debug
    if(!success) {
        glGetProgramInfoLog(shader->ID, 512, NULL, infoLog);
        printf("ERROR : Shader linking Failed %s\n",infoLog);
    }

    // delete the shaders as they're linked into our program now and no longer necessary
    glDeleteShader(vertex);
    glDeleteShader(fragment);
}

// use/activate the shader
void use(const SHADER *shader) {
    glUseProgram(shader->ID);
}

void setInt(const SHADER *shader, const char *name, const int value) {
    glUniform1i(glGetUniformLocation(shader->ID, name), value);
}

void setFloat(const SHADER *shader, const char *name, const float value) {
    glUniform1f(glGetUniformLocation(shader->ID, name), value);
}

void setVec2(const SHADER *shader, const char *name, const float v1,const float v2) {
    glUniform2f(glGetUniformLocation(shader->ID, name),v1, v2);
}

void setVec4(const SHADER *shader, const char *name, const float v1,const float v2, const float v3, const float v4) {
    glUniform4f(glGetUniformLocation(shader->ID, name),v1, v2, v3, v4);
}

void setMatrix4fv(const SHADER *shader, const char *name, const float *matrix) {
    glUniformMatrix4fv(glGetUniformLocation(shader->ID, name), 1, GL_FALSE, matrix);
}
