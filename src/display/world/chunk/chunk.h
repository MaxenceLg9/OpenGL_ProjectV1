//
// Created by maxence on 07/05/25.
//

#ifndef CHUNK_H
#define CHUNK_H

# define CHUNK_SIZE 64

#include <glm.hpp>

#include "../World.h"
#include "../../../math/math.h"
#include "glad/glad.h"
#include <map>
#include <memory>
#include <vector>
#include <mutex>    // for std::mutex
#include "../../model/mesh/shader/shader.h"
#include "../../model/mesh/mesh.h"
#include "../../model/mesh/ChunkMesh.h"




class World;

class ChunkMesh;

class Chunk {
public:
    Chunk(glm::ivec3 vec, std::map<glm::ivec3, Chunk *,IVec3Compare> *map, std::mutex *lock);

    ~Chunk();

    void render() const;

    void build_mesh(const World& world, glm::ivec3 chunkPos);

    uint16_t getBlockAt(glm::ivec3 blockPos) const;

private:
    uint16_t blocks[CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE];
    ChunkMesh *mesh;

    void generate_chunk(glm::ivec3 vec, std::map<glm::ivec3, Chunk *, IVec3Compare> *map, std::mutex *lock);
};

#endif //CHUNK_H
