//
// Created by Sinis on 08/05/2025.
//

#ifndef MESH_H
#define MESH_H

#include <vector>
#include "shader/shader.h"
#include "glm.hpp"

typedef struct {
    glm::vec3 Position;
    glm::vec3 Normal;
    glm::vec2 TexCoords;
} VERTEX;

typedef struct {
    unsigned int id;
    char* type;
    int code;
} TEXTURE;


class Mesh {
public:
    Mesh(std::vector<VERTEX> vertices, std::vector<unsigned int> indices, std::vector<TEXTURE> textures);
    void draw(Shader shader);
    int loadTextures(const char* filename, unsigned int tCode,const char *name);
    void freeMesh();

private:
    unsigned int VAO, VBO, EBO;
    std::vector<VERTEX> vertices;
    std::vector<unsigned int> indices;
    std::vector<TEXTURE> textures;



    void setupMesh();

    static void initTextures();
};

#endif //MESH_H
