//
// Created by maxence on 03/05/25.
//

#include "textures.h"
#include "stb_image.h"
#include <glad/glad.h>

int initTextures() {
    // set the texture wrapping/filtering options (on the currently bound texture object)
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_REPEAT);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_REPEAT);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST);
    return 1;
}


int loadTextures(unsigned int *texture, const char* filename, const unsigned int tCode) {
    glGenTextures(1, texture);
    glActiveTexture(tCode);
    glBindTexture(GL_TEXTURE_2D, *texture);
    initTextures();
    // load and generate the texture
    int width, height, nrChannels;
    stbi_set_flip_vertically_on_load(1);
    unsigned char *data = stbi_load(filename, &width, &height, &nrChannels, 0);
    printf("width: %d height: %d channels: %d\n", width, height, nrChannels);
    if (data)
    {
        GLint format = GL_RGB;
        if (nrChannels == 4)
            format = GL_RGBA;
        else if (nrChannels == 1)
            format = GL_RED;

        glTexImage2D(GL_TEXTURE_2D, 0, format, width, height, 0, format, GL_UNSIGNED_BYTE, data);
        glGenerateMipmap(GL_TEXTURE_2D);
    }
    else
    {
        printf("Failed to load texture\n");
        return -1;
    }
    stbi_image_free(data);
    return 0;
}
