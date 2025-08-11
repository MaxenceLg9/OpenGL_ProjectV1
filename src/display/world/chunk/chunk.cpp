//
// Created by maxence on 07/05/25.
//

#include "chunk.h"
#include "ext/matrix_transform.hpp"
#include "gtc/type_ptr.hpp"
#include "../../../logs/Logs.h"
#include "../../model/mesh/vertex/Vertex.h"

#include <gtc/noise.hpp>
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
        for (int y = 0; y < CHUNK_SIZE; y++) {
            for (int z = 0; z < CHUNK_SIZE; z++) {
                blocks[x * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + z] = (uint16_t) generate_block(glm::ivec3(x, y, z) + chunkPos * CHUNK_SIZE);
            }
        }
    }
    // printf("Locking lock and adding the chunk to the map\n");
    // printf("Unlocking lock\n");
    Logs::debug("Chunk created in " + std::to_string(time(nullptr) - t) + "seconds");
}

int Chunk::generate_block(glm::ivec3 blockPos) {
    const float amplitude = 50.0f;
    float ret = 0.0;
    float frequency = 0.01f;
    for (int i = 0; i < 2; i++) {
        ret += Utils::alpha(ret, i) * glm::perlin(glm::vec3((float) blockPos.x * frequency, (float) blockPos.z * frequency, 0.0));
        frequency *= 2.0;
    }
    int block = (blockPos.y < amplitude * ret + 100.0f) ? 1 : 0;
    return block; // Scale and offset the noise to fit in the range of 0-20
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