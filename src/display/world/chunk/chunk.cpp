//
// Created by maxence on 07/05/25.
//

#include "chunk.h"
#include "ext/matrix_transform.hpp"
#include "gtc/type_ptr.hpp"
#include "../../../logs/Logs.h"
#include "../../model/mesh/vertex/Vertex.h"

#include <gtc/noise.hpp>


Chunk::Chunk() {
    time_t t = time(nullptr);
    for (int x = 0; x < CHUNK_SIZE; x++) {
        for (int y = 0; y < CHUNK_SIZE; y++) {
            for (int z = 0; z < CHUNK_SIZE; z++) {
//                if (glm::simplex(glm::vec3(x,y,z) * 0.1f) > 0.2f)
//                    blocks[x * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + z] = 1;
//                else
//                    blocks[x * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + z] = 0;
                blocks[x * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + z] = (int) glm::simplex(glm::vec3(x,y,z) * 0.1f) + 0.9;
                // blocks[x * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + z] = 1;
            }
        }
    }
    printf("Chunk created in %lld seconds\n", time(nullptr) - t);
}

Chunk::~Chunk() {
    printf("Releasing Mesh %p\n",mesh);
    delete mesh;
}

void Chunk::render() const {
    mesh->draw();
}

int Chunk::getBlockAt(const glm::ivec3 blockPos) const {
    if (blockPos.x < 0 || blockPos.x >= CHUNK_SIZE || blockPos.y < 0 || blockPos.y >= CHUNK_SIZE || blockPos.z < 0 || blockPos.z >= CHUNK_SIZE) {
        return 0; // out of bounds
    }
    return blocks[(int) blockPos.x * CHUNK_SIZE * CHUNK_SIZE + (int) blockPos.y * CHUNK_SIZE + (int) blockPos.z];
}

void Chunk::build_mesh(const World& world, glm::ivec3 chunkPos) {
    mesh = new ChunkMesh(world, chunkPos, blocks);
}