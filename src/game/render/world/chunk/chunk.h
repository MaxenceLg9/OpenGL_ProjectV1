//
// Created by maxence on 07/05/25.
//

#ifndef CHUNK_H
#define CHUNK_H

# define CHUNK_SIZE 64

#include "glm.hpp"

#include "../World.h"
#include "GLAD/glad.h"
#include <map>
#include <memory>
#include <vector>
#include <mutex>    // for std::mutex
#include "../../../render/mesh/shader/shader.h"
#include "../../../render/mesh/ChunkMesh.h"




class World;

class ChunkMesh;

class Chunk {
public:
    Chunk(glm::ivec3 chunkPos, World *world);

    ~Chunk();

    ChunkMesh* build_mesh();

    uint16_t getBlockAt(glm::ivec3 blockPos) const;

private:
    uint16_t blocks[CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE];
    World *world;
    glm::ivec3 chunkPos;

    void generate_chunk();

    static int generate_block(int y);

    glm::ivec3 getChunkPos() const;
};

#endif //CHUNK_H
