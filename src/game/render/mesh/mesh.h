//
// Created by Sinis on 08/05/2025.
//

#ifndef MESH_H
#define MESH_H

#include <string>
#include <vector>
#include "shader/shader.h"
#include "glm.hpp"
#include "texture/textures.h"
#include "vertex/Vertex.h"


class Mesh {
public:
    Mesh(std::vector<VERTEX> vertices, std::vector<unsigned int> indices, std::vector<TEXTURE> textures);
    void draw(const Shader &shader);
    int loadTextures(const char *filename, unsigned int tCode, const std::string &name);
    ~Mesh();

private:
    unsigned int VAO, VBO, EBO;
    std::vector<VERTEX> vertices;
    std::vector<unsigned int> indices;
    std::vector<TEXTURE> textures;



    void setupMesh();

    static void initTextures();
};

#endif //MESH_H