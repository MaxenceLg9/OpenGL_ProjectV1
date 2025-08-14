//
// Created by Sinis on 12/08/2025.
//

#include "TextureArray.h"
#include "stb_image.h"
#define STB_IMAGE_RESIZE_IMPLEMENTATION
#include "stb_image_resize2.h"
#include "../../../../logs/Logs.h"
#include <string>

TextureArray::TextureArray(const std::string &type) : type(type), texture(0) {
    glGenTextures(1,&texture);
    glBindTexture(GL_TEXTURE_2D_ARRAY,texture);
    glTexParameteri(GL_TEXTURE_2D_ARRAY, GL_TEXTURE_WRAP_S, GL_REPEAT);
    glTexParameteri(GL_TEXTURE_2D_ARRAY, GL_TEXTURE_WRAP_T, GL_REPEAT);
    glTexParameteri(GL_TEXTURE_2D_ARRAY, GL_TEXTURE_MIN_FILTER, GL_NEAREST);
    glTexParameteri(GL_TEXTURE_2D_ARRAY, GL_TEXTURE_MAG_FILTER, GL_NEAREST);
// Allocate the storage.
    glTexStorage3D(GL_TEXTURE_2D_ARRAY, 1, GL_RGBA8, TEXTURE_SIZE, TEXTURE_SIZE, TEXTURE_ARRAY_SIZE);
}

void TextureArray::addTexture(const std::string& filename, int index){
    int width, height, nrChannels;
    stbi_set_flip_vertically_on_load(1);
    unsigned char *data = stbi_load(filename.c_str(), &width, &height, &nrChannels, STBIR_RGBA);
    if(width != height) {
        Logs::debug("Texture " + filename + " is not square");
        return;
    }
    if(width != TEXTURE_SIZE || height != TEXTURE_SIZE) {
        unsigned char *resizedData = (unsigned char *) malloc(TEXTURE_SIZE * TEXTURE_SIZE * 4);
        stbir_resize(data, width, height, 0, resizedData, TEXTURE_SIZE, TEXTURE_SIZE, 0, STBIR_RGBA, STBIR_TYPE_UINT8_SRGB, STBIR_EDGE_CLAMP,STBIR_FILTER_POINT_SAMPLE );
        stbi_image_free(data);
        data = resizedData;

        Logs::debug("Image resized from : " + std::to_string(width) + "x" + std::to_string(height) + "x" + std::to_string(nrChannels) +  " to : " + std::to_string(width) + "x" + std::to_string(height) + "x" + std::to_string(nrChannels));
    }
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_REPEAT);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_REPEAT);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST);
    glTexSubImage3D(GL_TEXTURE_2D_ARRAY, 0, 0, 0, index, TEXTURE_SIZE, TEXTURE_SIZE, 1, GL_RGBA, GL_UNSIGNED_BYTE, data);

    stbi_image_free(data);
}

void TextureArray::use_textures(const Shader &shader) const {
    glActiveTexture(GL_TEXTURE0);
    shader.setInt(this->type.c_str(), 0);
    glBindTexture(GL_TEXTURE_2D_ARRAY, this->texture);
}