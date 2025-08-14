//
// Created by maxence on 07/05/25.
//

#include "chunk.h"
#include "ext/matrix_transform.hpp"
#include "gtc/type_ptr.hpp"
#include "../../../logs/Logs.h"
#include "../../../display/model/mesh/vertex/Vertex.h"
#include "../block/block.h"

#include <cmath>
#include "gtc/noise.hpp"
#include <thread>


Chunk::Chunk(const glm::ivec3 chunkPos, World *world) {
    this->world = world;
    this->chunkPos = chunkPos;
    generate_chunk();
}

Chunk::~Chunk() {
    printf("Releasing Mesh %p\n",mesh);
    delete mesh;
    mesh = nullptr;
    world = nullptr;
}

void Chunk::generate_chunk(){
    const time_t t = time(nullptr);
    for (int x = 0; x < CHUNK_SIZE; x++) {
        for (int z = 0; z < CHUNK_SIZE; z++) {
            int blockX = x + chunkPos.x * CHUNK_SIZE, blockZ = z + chunkPos.z * CHUNK_SIZE;
            int maxH = Utils::noised_terrain_default(blockX,blockZ) * 200.f + 200.f;
            int localMaxHeight = maxH - chunkPos.y * CHUNK_SIZE;
//            Logs::debug("MaxH: " + std::to_string(maxH));
            for (int y = 0; y < localMaxHeight && y < CHUNK_SIZE; y++) {
                blocks[x * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + z] = (uint16_t) generate_block(y + chunkPos.y * CHUNK_SIZE);
            }
            for (int y = Utils::max(localMaxHeight,0) ; y < CHUNK_SIZE; y++) {
                blocks[x * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + z] = AIR;
            }
        }
    }
    // printf("Locking buildLock and adding the chunk to the map\n");
    // printf("Unlocking buildLock\n");
//    Logs::debug("Chunk created in " + std::to_string(time(nullptr) - t) + "seconds");
}

int Chunk::generate_block(int y) {
    if (y < 100)
        return DEEPSLATE; // Deepslate
    if (y < 200 || y > 400)
        return STONE; // Stone
    else
        return DIRT; // Dirt;
}

glm::ivec3 Chunk::getChunkPos() const {
    return chunkPos;
}

void Chunk::render() const {
    mesh->draw();
}

uint16_t Chunk::getBlockAt(const glm::ivec3 blockPos) const {
    if (blockPos.x < 0 || blockPos.x >= CHUNK_SIZE || blockPos.y < 0 || blockPos.y >= CHUNK_SIZE || blockPos.z < 0 || blockPos.z >= CHUNK_SIZE) {
        return 0; // out of bounds
    }
    return blocks[(int) blockPos.x * CHUNK_SIZE * CHUNK_SIZE + (int) blockPos.y * CHUNK_SIZE + (int) blockPos.z];
}

void Chunk::build_mesh() {
    mesh = new ChunkMesh(*world, chunkPos, blocks);
}

void Chunk::link_mesh(){
    if(mesh == nullptr) {
        Logs::debug("Mesh is null, building mesh");
    }
    mesh->link();
}