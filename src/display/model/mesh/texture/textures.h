//
// Created by maxence on 03/05/25.
//

#ifndef TEXTURES_H
#define TEXTURES_H

#include <string>
#include <glad/glad.h>
#include "../shader/shader.h"

typedef struct {
    unsigned int id;
    std::string type;
    int code;
} TEXTURE;

class Texture {
public:
    Texture(const char* filename, unsigned int tCode,const std::string& name);
    ~Texture();
    void use_textures(const Shader &shader);
private:
    std::string type;
    int code;
    uint id;
};

#endif //TEXTURES_H
