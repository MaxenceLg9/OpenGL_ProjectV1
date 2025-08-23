//
// Created by Sinis on 12/08/2025.
//

#ifndef TEXTUREARRAY_H
#define TEXTUREARRAY_H

#define TEXTURE_ARRAY_SIZE 256
#define TEXTURE_SIZE 64

#include <glad/glad.h>
#include <string>

#include "../../../../utils/logs/Logs.h"
#include "../shader/shader.h"

class TextureArray {
public:
    explicit TextureArray(const std::string &type);
    ~TextureArray();
    void addTexture(const std::string& filename, int index);

    void use_textures(const Shader &shader) const;

private:
    unsigned int texture;
    std::string type;

};


#endif //TEXTUREARRAY_H
