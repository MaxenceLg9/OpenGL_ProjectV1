#include "textures.h"
#define STB_IMAGE_IMPLEMENTATION
#include "stb_image.h"
#include <regex>

Texture::Texture(const char *filename, unsigned int tCode, const std::string &name) {
    this->type = name;
    this->code = (int) tCode;
    glGenTextures(1, &this->id);
    glActiveTexture(tCode);
    glBindTexture(GL_TEXTURE_2D, this->id);
    // set the texture wrapping/filtering options (on the currently bound texture object)
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_REPEAT);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_REPEAT);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST);
    // load and generate the texture
    int width, height, nrChannels;
    stbi_set_flip_vertically_on_load(1);
    unsigned char *data = stbi_load(filename, &width, &height, &nrChannels, 0);
    // printf("Loaded image, Width: %d Height: %d Channels: %d\n", width, height, nrChannels);
    if (data) {
        GLint format = GL_RGB;
        if (nrChannels == 4)
            format = GL_RGBA;
        else if (nrChannels == 1) {
            format = GL_RED;
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_SWIZZLE_R, GL_RED);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_SWIZZLE_G, GL_RED);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_SWIZZLE_B, GL_RED);
        }
        glTexImage2D(GL_TEXTURE_2D, 0, format, width, height, 0, format, GL_UNSIGNED_BYTE, data);
        glGenerateMipmap(GL_TEXTURE_2D);
        stbi_image_free(data);
    } else {
        printf("Failed to load texture\n");
        throw std::runtime_error("Failed to load texture");
    }
}

Texture::~Texture() {
    glDeleteTextures(1, &this->id);
}

void Texture::use_textures(const Shader &shader) {
    glActiveTexture(this->code); // active proper texture unit before binding
    // now set the sampler to the correct texture unit
    shader.setInt((this->type).c_str(), GL_TEXTURE0 - this->code);
    // and finally bind the texture
    glBindTexture(GL_TEXTURE_2D, this->id);
}
