//
// Created by Sinis on 31/05/2025.
//

#ifndef CHUNKMESH_H
#define CHUNKMESH_H

#include "../world/World.h"

class World;

class ChunkMesh {
public:
    explicit ChunkMesh(const World &world, glm::ivec3 chunkPos, uint16_t *blocks);

    ~ChunkMesh() = default;

    void buildMesh(const World &world, glm::ivec3 chunkPos, const uint16_t *blocks);

    /**
     * After building the chunk mesh asynchronously, needs to be linked with openGL from main thread
     * Links all chunk meshes that are ready to be linked.
     */
    void link();

    bool is_linked() const;

    void draw() const;

private:

    int addData(const uint64_t *v, int index) const;

    void bindData() const;

    void setupMesh();

    static bool isVoid(glm::ivec3 blockPos, const uint16_t *blocks, const World &world, glm::ivec3 chunkPos);

    unsigned int VAO, VBO, EBO, nbIndices;
    std::vector<uint32_t> *vertices;
    std::vector<unsigned int> *indices;

    bool linked;
};

#endif //CHUNKMESH_H
