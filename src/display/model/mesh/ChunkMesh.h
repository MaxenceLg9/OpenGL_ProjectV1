//
// Created by Sinis on 31/05/2025.
//

#ifndef CHUNKMESH_H
#define CHUNKMESH_H

#include "../../../game/world/World.h"

class World;

class ChunkMesh {
public:
    explicit ChunkMesh(const World &world, glm::ivec3 chunkPos, uint16_t *blocks);

    ~ChunkMesh() = default;

    void buildMesh(const World &world, glm::ivec3 chunkPos, const uint16_t *blocks);

    void draw() const;

    void link();

private:

    int addData(uint64_t *v, int index);

    void bindData() const;

    void setupMesh();

    static bool isVoid(glm::ivec3 blockPos, const uint16_t *blocks, const World &world, glm::ivec3 chunkPos);

    unsigned int VAO, VBO, EBO, nbIndices;
    std::vector<uint32_t> *vertices;
    std::vector<unsigned int> *indices;
};

#endif //CHUNKMESH_H
