//
// Created by Sinis on 08/05/2025.
//

#include "mesh.h"
#include "stb_image.h"
#include <utility>
#include <vector>
#include <cstdio>
#include <string>

#include "../../../utils/logs/Logs.h"
#include "glad/glad.h"

Mesh::Mesh(std::vector<VERTEX> vertices, std::vector<unsigned int> indices, std::vector<TEXTURE> textures) {
    this->vertices = std::move(vertices);
    this->indices = std::move(indices);
    this->textures = std::move(textures);
    this->VAO = 0, this->VBO = 0, this->EBO = 0;
    setupMesh();
}

void Mesh::setupMesh() {
    glCreateBuffers(1, &VBO);
    glNamedBufferData(VBO,vertices.size() * sizeof(VERTEX), &vertices[0], GL_STATIC_DRAW);

    glCreateBuffers(1, &EBO);
    glNamedBufferData(EBO, indices.size() * sizeof(unsigned int),&indices[0],GL_STATIC_DRAW);

    glCreateVertexArrays(1, &VAO);
    glVertexArrayVertexBuffer(VAO, 0, VBO, 0, sizeof(VERTEX));
    glVertexArrayElementBuffer(VAO, EBO);

    //Enable vertex attributes (location = ?)
    glEnableVertexArrayAttrib(VAO, 0);
    glEnableVertexArrayAttrib(VAO, 1);
    glEnableVertexArrayAttrib(VAO, 2);

    glVertexArrayAttribFormat(VAO,0,3,GL_FLOAT, GL_FALSE, offsetof(VERTEX, Position));
    glVertexArrayAttribFormat(VAO,1,3,GL_FLOAT, GL_FALSE,offsetof(VERTEX, Normal));
    glVertexArrayAttribFormat(VAO,2,2,GL_FLOAT, GL_FALSE,offsetof(VERTEX, TexCoords));

    glVertexArrayAttribBinding(VAO,0,0);
    glVertexArrayAttribBinding(VAO,1,0);
    glVertexArrayAttribBinding(VAO,2,0);

    Logs::debug("Mesh created with VBO: " + std::to_string(VBO) + ", EBO: " + std::to_string(EBO) + ", VAO: " + std::to_string(VAO));
}

void Mesh::draw(const Shader& shader) {     // render the mesh
    // bind appropriate textures
    unsigned int diffuseNr  = 1;
    unsigned int specularNr = 1;
    unsigned int normalNr   = 1;
    unsigned int heightNr   = 1;
    for(unsigned int i = 0; i < textures.size(); i++)
    {
        glActiveTexture(textures[i].code); // active proper texture unit before binding
        // retrieve texture number (the N in diffuse_textureN)
        std::string number;
        std::string name = textures[i].type;
        if(name == "texture_diffuse")
            number = std::to_string(diffuseNr++);
        else if(name == "texture_specular")
            number = std::to_string(specularNr++); // transfer unsigned int to string
        else if(name == "texture_normal")
            number = std::to_string(normalNr++); // transfer unsigned int to string
        else if(name == "texture_height")
            number = std::to_string(heightNr++); // transfer unsigned int to string

        // now set the sampler to the correct texture unit
        shader.setInt((name).c_str(), i);
        // and finally bind the texture
        glBindTexture(GL_TEXTURE_2D, textures[i].id);
    }

    // draw mesh
    glBindVertexArray(VAO);
    glDrawElementsBaseVertex(GL_TRIANGLES, (int) indices.size(), GL_UNSIGNED_INT, (void *) 0, 0);
    glBindVertexArray(0);


    // always good practice to set everything back to defaults once configured.
    glActiveTexture(GL_TEXTURE0);
}

int Mesh::loadTextures(const char* filename, unsigned int tCode,const std::string& name){
    TEXTURE texture;
    texture.type = name;
    texture.code = (int) tCode;
    glGenTextures(1, &texture.id);
    glActiveTexture(tCode);
    glBindTexture(GL_TEXTURE_2D, texture.id);
    Mesh::initTextures();
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
        return -1;
    }
    this->textures.push_back(texture);
    return 0;
}

void Mesh::initTextures() {
    // set the texture wrapping/filtering options (on the currently bound texture object)
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_REPEAT);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_REPEAT);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST);
}

Mesh::~Mesh() {
    glDeleteVertexArrays(1, &VAO);
    glDeleteBuffers(1, &VBO);
    glDeleteBuffers(1, &EBO);
    for(auto & texture : textures){
        glDeleteTextures(1, &texture.id);
    }
}
