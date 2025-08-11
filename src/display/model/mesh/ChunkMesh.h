//
// Created by Sinis on 31/05/2025.
//

#ifndef CHUNKMESH_H
#define CHUNKMESH_H

#include "../../world/World.h"

class World;

class ChunkMesh {
public:
    explicit ChunkMesh(const World &world, glm::ivec3 chunkPos, uint16_t *blocks);

    ~ChunkMesh() = default;

    void buildMesh(const World &world, glm::ivec3 chunkPos, const uint16_t *blocks);

    void draw() const;

private:
    unsigned int VAO, VBO, EBO, nbIndices;

    static int addData(std::vector<uint32_t> *vertex, std::vector<unsigned int> *indices, uint64_t *v, int index);

    void bindData(std::vector<uint32_t> *vertices, std::vector<unsigned int> *indices) const;

    void setupMesh();

    static bool isVoid(glm::ivec3 blockPos, const uint16_t *blocks, const World &world, glm::ivec3 chunkPos);
};

#endif //CHUNKMESH_H
