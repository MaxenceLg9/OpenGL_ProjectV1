//
// Created by Sinis on 31/05/2025.
//

#include "ChunkMesh.h"
#include "vertex/Vertex.h"
#include "../../../logs/Logs.h"

ChunkMesh::ChunkMesh(const World &world, glm::ivec3 chunkPos, uint16_t *blocks) {
    setupMesh();
    buildMesh(world, chunkPos, blocks);
}

void ChunkMesh::setupMesh() {
    glCreateBuffers(1, &VBO);
    glCreateBuffers(1, &EBO);

    glCreateVertexArrays(1, &VAO);
    glVertexArrayVertexBuffer(VAO, 0, VBO, 0, 8);
    glVertexArrayElementBuffer(VAO, EBO);

    glEnableVertexArrayAttrib(VAO, 0);
    glEnableVertexArrayAttrib(VAO, 1);

    glVertexArrayAttribFormat(VAO,0,1,GL_UNSIGNED_INT, GL_FALSE, 0);
    glVertexArrayAttribFormat(VAO,1,1,GL_UNSIGNED_INT, GL_FALSE, 4);

    glVertexArrayAttribBinding(VAO,0,0);
    glVertexArrayAttribBinding(VAO,1,0);


    printf("VBO: %u, EBO: %u, VAO: %u\n", VBO, EBO, VAO);
    Logs::log("INFO", "ChunkMesh created with VBO: " + std::to_string(VBO) + ", EBO: " + std::to_string(EBO) + ", VAO: " + std::to_string(VAO));
}

void ChunkMesh::bindData(std::vector<uint32_t> &vertices, std::vector<unsigned int> &indices) const{
    glNamedBufferData(VBO,vertices.size() * sizeof(uint32_t), &vertices[0], GL_STATIC_DRAW);
    glNamedBufferData(EBO, indices.size() * sizeof(unsigned int),&indices[0],GL_STATIC_DRAW);
}

void ChunkMesh::buildMesh(const World &world, glm::ivec3 chunkPos, const uint16_t *blocks) {
    std::vector<uint32_t> vertices;
    vertices.reserve(CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE * 6 * 4 * 2); // 6 faces, 4 vertices per face
    std::vector<unsigned int> indices;
    indices.reserve(CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE * 6 * 6); // 6 faces, 6 nbIndices per face
    int index = 0;
    for (int x = 0; x < CHUNK_SIZE; x++) {
        for (int y = 0; y < CHUNK_SIZE; y++) {
            for (int z = 0; z < CHUNK_SIZE; z++) {
                uint16_t voxel_id = blocks[x * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + z];

                if (voxel_id == 0) continue; // skip empty blocks
                uint64_t v[4];
                //front face
                if (isVoid(glm::vec3(x, y, z + 1), blocks, world, chunkPos)) {

                    v[0] = Vertex::packData(voxel_id, glm::ivec3(x, y, z + 1), glm::vec3(0.0f, 0.0f, 1.0f), 0);
                    v[1] = Vertex::packData(voxel_id, glm::vec3(x, y + 1, z + 1), glm::vec3(0.0f, 0.0f, 1.0f), 1);
                    v[2] = Vertex::packData(voxel_id, glm::vec3(x + 1, y + 1, z + 1), glm::vec3(0.0f, 0.0f, 1.0f), 3);
                    v[3] = Vertex::packData(voxel_id, glm::vec3(x + 1, y, z + 1), glm::vec3(0.0f, 0.0f, 1.0f), 2);

                    index = addData(vertices, indices, v, index);
                }
                // back face
                if (isVoid(glm::vec3(x, y, z - 1), blocks, world, chunkPos)) {

                    v[0] = Vertex::packData(voxel_id, glm::ivec3(x, y, z), glm::vec3(0.0f, 0.0f, -1.0f), 2);
                    v[1] = Vertex::packData(voxel_id, glm::vec3(x + 1, y, z), glm::vec3(0.0f, 0.0f, -1.0f), 0);
                    v[2] = Vertex::packData(voxel_id, glm::vec3(x + 1, y + 1, z), glm::vec3(0.0f, 0.0f, -1.0f), 1);
                    v[3] = Vertex::packData(voxel_id, glm::vec3(x, y + 1, z), glm::vec3(0.0f, 0.0f, -1.0f), 3);

                    index = addData(vertices, indices, v, index);
                }
                //top face
                if (isVoid(glm::vec3(x, y + 1, z), blocks, world, chunkPos)) {

                    v[0] = Vertex::packData(voxel_id, glm::vec3(x, y + 1, z), glm::vec3(0.0f, 0.0f, -1.0f), 2);
                    v[1] = Vertex::packData(voxel_id, glm::vec3(x + 1, y + 1, z), glm::vec3(0.0f, 0.0f, -1.0f), 0);
                    v[2] = Vertex::packData(voxel_id, glm::vec3(x + 1, y + 1, z + 1), glm::vec3(0.0f, 0.0f, -1.0f), 1);
                    v[3] = Vertex::packData(voxel_id, glm::vec3(x, y + 1, z + 1), glm::vec3(0.0f, 0.0f, -1.0f), 3);

                    index = addData(vertices, indices, v, index);
                }
                // bottom face
                if (isVoid(glm::vec3(x, y - 1, z), blocks, world, chunkPos)) {

                    v[0] = Vertex::packData(voxel_id, glm::vec3(x, y, z), glm::vec3(0.0f, 0.0f, -1.0f), 1);
                    v[1] = Vertex::packData(voxel_id, glm::vec3(x + 1, y, z), glm::vec3(0.0f, 0.0f, -1.0f), 0);
                    v[2] = Vertex::packData(voxel_id, glm::vec3(x + 1, y, z + 1), glm::vec3(0.0f, 0.0f, -1.0f), 2);
                    v[3] = Vertex::packData(voxel_id, glm::vec3(x, y, z + 1), glm::vec3(0.0f, 0.0f, -1.0f), 3);

                    index = addData(vertices, indices, v, index);
                }

                // right face
                if (isVoid(glm::vec3(x + 1, y, z), blocks, world, chunkPos)) {

                    v[0] = Vertex::packData(voxel_id, glm::vec3(x + 1, y, z), glm::vec3(0.0f, 0.0f, -1.0f), 2);
                    v[1] = Vertex::packData(voxel_id, glm::vec3(x + 1, y + 1, z), glm::vec3(0.0f, 0.0f, -1.0f), 3);
                    v[2] = Vertex::packData(voxel_id, glm::vec3(x + 1, y + 1, z + 1), glm::vec3(0.0f, 0.0f, -1.0f), 1);
                    v[3] = Vertex::packData(voxel_id, glm::vec3(x + 1, y, z + 1), glm::vec3(0.0f, 0.0f, -1.0f), 0);

                    index = addData(vertices, indices, v, index);
                }

                // left face
                if (isVoid(glm::vec3(x - 1, y, z), blocks, world, chunkPos)) {

                    v[0] = Vertex::packData(voxel_id, glm::vec3(x, y, z), glm::vec3(0.0f, 0.0f, -1.0f), 0);
                    v[1] = Vertex::packData(voxel_id, glm::vec3(x, y + 1, z), glm::vec3(0.0f, 0.0f, -1.0f), 1);
                    v[2] = Vertex::packData(voxel_id, glm::vec3(x, y + 1, z + 1), glm::vec3(0.0f, 0.0f, -1.0f), 3);
                    v[3] = Vertex::packData(voxel_id, glm::vec3(x, y, z + 1), glm::vec3(0.0f, 0.0f, -1.0f), 2);

                    index = addData(vertices, indices, v, index);
                }
            }
        }
    }
    this->nbIndices = indices.size();
    printf("Size %llu : %llu\n", vertices.size(), indices.size());
    bindData(vertices, indices);
}

bool ChunkMesh::isVoid(glm::ivec3 blockPos, const uint16_t *blocks, const World &world, glm::ivec3 chunkPos) {
    if (blockPos.x < 0 || blockPos.x >= CHUNK_SIZE ||
        blockPos.y < 0 || blockPos.y >= CHUNK_SIZE ||
        blockPos.z < 0 || blockPos.z >= CHUNK_SIZE) {
        return world.getBlockAt(chunkPos * CHUNK_SIZE + blockPos) == 0;
    }
    return blocks[blockPos.x * CHUNK_SIZE * CHUNK_SIZE + blockPos.y * CHUNK_SIZE + blockPos.z] == 0;
}

void ChunkMesh::draw() const {
    glBindVertexArray(VAO);
    glDrawElementsBaseVertex(GL_TRIANGLES, (int) nbIndices, GL_UNSIGNED_INT, (void *) 0, 0);
    glBindVertexArray(0);
}

int ChunkMesh::addData(std::vector<uint32_t> &vertex, std::vector<unsigned int> &indices, uint64_t *v, int index) {

    for (int i = 0; i < 4; ++i) {
        vertex.push_back((uint32_t)(v[i] >> 32));        // High 32 bits
        vertex.push_back((uint32_t)(v[i] & 0xFFFFFFFF)); // Low 32 bits
    }

    indices.push_back(index);
    indices.push_back(index + 1);
    indices.push_back(index + 2);
    indices.push_back(index);
    indices.push_back(index + 2);
    indices.push_back(index + 3);

    return index + 4;
}

