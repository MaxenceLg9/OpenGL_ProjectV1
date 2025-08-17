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
#include "../../../render/model/mesh/shader/shader.h"
#include "../../../render/model/mesh/mesh.h"
#include "../../../render/model/mesh/ChunkMesh.h"




class World;

class ChunkMesh;

class Chunk {
public:
    Chunk(glm::ivec3 chunkPos, World *world);

    ~Chunk();

    void render() const;

    void build_mesh();

    uint16_t getBlockAt(glm::ivec3 blockPos) const;

    void link_mesh();

private:
    uint16_t blocks[CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE];
    ChunkMesh *mesh;
    World *world;
    glm::ivec3 chunkPos;

    void generate_chunk();

    int generate_block(int y);

    glm::ivec3 getChunkPos() const;
};

#endif //CHUNK_H
