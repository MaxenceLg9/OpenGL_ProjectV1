//
// Created by maxence on 07/05/25.
//

#ifndef CHUNK_H
#define CHUNK_H

# define CHUNK_SIZE 64

#include "glm.hpp"

#include "../World.h"
#include "../../model/mesh/shader/shader.h"
#include "../../model/mesh/mesh.h"

class World;

class Chunk {
public:
    explicit Chunk(const World &world, glm::ivec3 chunkPos);

    ~Chunk();

    void render(const Shader& shader, const glm::mat4 & p_v, glm::vec3 playerPos) const;

    static int addData(std::vector<VERTEX> &vertex, std::vector<unsigned int> &indices, VERTEX *v, int index);

    void build_mesh(const uint8_t blocks[], const World& world, glm::ivec3 chunkPos);

    static bool isVoid(glm::ivec3 blockPos, const uint8_t blocks[], const World& world, glm::ivec3 chunkPos);

    int getBlockAt(glm::ivec3 pos) const;

private:
    uint8_t blocks[CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE];
    std::vector<VERTEX> vertexdata;
    std::vector<unsigned int> indices;
    Mesh *mesh;
};

#endif //CHUNK_H
