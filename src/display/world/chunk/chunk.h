//
// Created by maxence on 07/05/25.
//

#ifndef CHUNK_H
#define CHUNK_H

# define CHUNK_SIZE 64

#include "glm.hpp"
#include <memory>
#include "../../model/mesh/shader/shader.h"
#include "../block/block.h"
#include "../../model/mesh/mesh.h"


class Chunk {
public:
    Chunk();
    ~Chunk();

    void render(const Shader& shader, const glm::mat4 & p_v, glm::vec3 pos) const;

    static int addData(std::vector<VERTEX> &vertex, std::vector<unsigned int> &indices, VERTEX *v, int index);

    void build_mesh(const uint8_t blocks[]);

    static bool isVoid(glm::vec3 pos, const uint8_t blocks[]);

private:
    uint8_t blocks[CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE];
    std::vector<VERTEX> vertexdata;
    std::vector<unsigned int> indices;
    Mesh *mesh;
};

#endif //CHUNK_H
