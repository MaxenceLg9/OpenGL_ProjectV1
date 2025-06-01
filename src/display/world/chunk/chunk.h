//
// Created by maxence on 07/05/25.
//

#ifndef CHUNK_H
#define CHUNK_H

# define CHUNK_SIZE 64

#include <glm.hpp>

#include "glad/glad.h"
#include "../World.h"
#include "../../model/mesh/shader/shader.h"
#include "../../model/mesh/mesh.h"
#include "../../model/mesh/ChunkMesh.h"


class World;

class ChunkMesh;

class Chunk {
public:
    Chunk();

    ~Chunk();

    void render() const;

    void build_mesh(const World& world, glm::ivec3 chunkPos);

    int getBlockAt(glm::ivec3 blockPos) const;

private:
    uint16_t blocks[CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE];
    ChunkMesh *mesh;
};

#endif //CHUNK_H
