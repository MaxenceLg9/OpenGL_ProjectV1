//
// Created by Sinis on 31/05/2025.
//

#include "ChunkMesh.h"
#include "vertex/Vertex.h"
#include "../../../utils/logs/Logs.h"

ChunkMesh::ChunkMesh(const World &world, glm::ivec3 chunkPos, uint16_t *blocks) {
    linked = false;
    buildMesh(world, chunkPos, blocks);
}

void ChunkMesh::link() {
    setupMesh();
    bindData();
    linked = true;
    delete vertices;
    delete indices;
}

bool ChunkMesh::is_linked() const
{
    return linked;
}

void ChunkMesh::setupMesh() {
    glCreateBuffers(1, &VBO);
    glCreateBuffers(1, &EBO);

    glCreateVertexArrays(1, &VAO);
    glVertexArrayVertexBuffer(VAO, 0, VBO, 0, 8);
    glVertexArrayElementBuffer(VAO, EBO);

    glEnableVertexArrayAttrib(VAO, 0);
    glEnableVertexArrayAttrib(VAO, 1);

    glVertexArrayAttribIFormat(VAO, 0, 1, GL_UNSIGNED_INT, 0);
    glVertexArrayAttribIFormat(VAO, 1, 1, GL_UNSIGNED_INT, 4);

    glVertexArrayAttribBinding(VAO,0,0);
    glVertexArrayAttribBinding(VAO,1,0);

    Logs::log("INFO", "ChunkMesh created with VBO: " + std::to_string(VBO) + ", EBO: " + std::to_string(EBO) + ", VAO: " + std::to_string(VAO));
}

void ChunkMesh::bindData() const{
    glNamedBufferData(VBO,vertices->size() * sizeof(uint32_t), vertices->data(), GL_STATIC_DRAW);
    glNamedBufferData(EBO, indices->size() * sizeof(unsigned int),indices->data(),GL_STATIC_DRAW);
}

void ChunkMesh::buildMesh(const World &world, glm::ivec3 chunkPos, const uint16_t *blocks) {
    vertices = new std::vector<uint32_t>();
    vertices->reserve(CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE * 6 * 4 * 2); // 6 faces, 4 vertices per face
    indices = new std::vector<unsigned int>();
    indices->reserve(CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE * 6 * 6); // 6 faces, 6 nbIndices per face
    int index = 0;
    for (int x = 0; x < CHUNK_SIZE; x++) {
        for (int y = 0; y < CHUNK_SIZE; y++) {
            for (int z = 0; z < CHUNK_SIZE; z++) {
                uint16_t voxel_id = blocks[x * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + z];

                if (voxel_id == 0) continue; // skip empty blocks
                uint64_t v[4];
                //front face
                if (isVoid(glm::vec3(x, y, z + 1), blocks, world, chunkPos)) {

                    v[0] = Vertex::packData(voxel_id, glm::ivec3(x, y, z + 1), 1, 0);
                    v[1] = Vertex::packData(voxel_id, glm::ivec3(x, y + 1, z + 1), 1, 1);
                    v[2] = Vertex::packData(voxel_id, glm::ivec3(x + 1, y + 1, z + 1), 1, 3);
                    v[3] = Vertex::packData(voxel_id, glm::ivec3(x + 1, y, z + 1), 1, 2);

                    index = addData(v, index);
                }
                // back face
                if (isVoid(glm::vec3(x, y, z - 1), blocks, world, chunkPos)) {

                    v[0] = Vertex::packData(voxel_id, glm::ivec3(x, y, z), 4, 2);
                    v[1] = Vertex::packData(voxel_id, glm::ivec3(x + 1, y, z), 4, 0);
                    v[2] = Vertex::packData(voxel_id, glm::ivec3(x + 1, y + 1, z), 4, 1);
                    v[3] = Vertex::packData(voxel_id, glm::ivec3(x, y + 1, z), 4, 3);

                    index = addData(v, index);
                }
                //top face
                if (isVoid(glm::vec3(x, y + 1, z), blocks, world, chunkPos)) {

                    v[0] = Vertex::packData(voxel_id, glm::ivec3(x, y + 1, z), 0, 2);
                    v[1] = Vertex::packData(voxel_id, glm::ivec3(x + 1, y + 1, z), 0, 0);
                    v[2] = Vertex::packData(voxel_id, glm::ivec3(x + 1, y + 1, z + 1), 0, 1);
                    v[3] = Vertex::packData(voxel_id, glm::ivec3(x, y + 1, z + 1), 0, 3);

                    index = addData(v, index);
                }
                // bottom face
                if (isVoid(glm::vec3(x, y - 1, z), blocks, world, chunkPos)) {

                    v[0] = Vertex::packData(voxel_id, glm::ivec3(x, y, z), 5, 1);
                    v[1] = Vertex::packData(voxel_id, glm::ivec3(x, y, z + 1), 5, 3);
                    v[2] = Vertex::packData(voxel_id, glm::ivec3(x + 1, y, z + 1), 5, 2);
                    v[3] = Vertex::packData(voxel_id, glm::ivec3(x + 1, y, z), 5, 0);

                    index = addData(v, index);
                }

                // right face
                if (isVoid(glm::vec3(x + 1, y, z), blocks, world, chunkPos)) {

                    v[0] = Vertex::packData(voxel_id, glm::ivec3(x + 1, y, z), 2, 2);
                    v[1] = Vertex::packData(voxel_id, glm::ivec3(x + 1, y, z + 1), 2, 0);
                    v[2] = Vertex::packData(voxel_id, glm::ivec3(x + 1, y + 1, z + 1), 2, 1);
                    v[3] = Vertex::packData(voxel_id, glm::ivec3(x + 1, y + 1, z), 2, 3);

                    index = addData(v, index);
                }

                // left face
                if (isVoid(glm::vec3(x - 1, y, z), blocks, world, chunkPos)) {

                    v[0] = Vertex::packData(voxel_id, glm::ivec3(x, y, z), 3, 0);
                    v[1] = Vertex::packData(voxel_id, glm::ivec3(x, y + 1, z), 3, 1);
                    v[2] = Vertex::packData(voxel_id, glm::ivec3(x, y + 1, z + 1), 3, 3);
                    v[3] = Vertex::packData(voxel_id, glm::ivec3(x, y, z + 1), 3, 2);

                    index = addData(v, index);
                }
            }
        }
    }
    this->nbIndices = indices->size();
    if (nbIndices == 0) {
//        Logs::debug("No vertices to draw");
        return;
    }
//    Logs::debug("Size " + std::to_string(vertices->size()) + " : " + std::to_string(indices->size()));
//    Logs::debug("Data bound to VBO and EBO");
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

int ChunkMesh::addData(const uint64_t *v, const int index) const
{

    for (int i = 0; i < 4; ++i) {
        vertices->push_back((uint32_t)(v[i] >> 32));        // High 32 bits
        vertices->push_back((uint32_t)(v[i] & 0xFFFFFFFF)); // Low 32 bits
    }

    indices->push_back(index);
    indices->push_back(index + 1);
    indices->push_back(index + 2);
    indices->push_back(index);
    indices->push_back(index + 2);
    indices->push_back(index + 3);

    return index + 4;
}

