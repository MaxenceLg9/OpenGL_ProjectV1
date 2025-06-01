//
// Created by Sinis on 31/05/2025.
//

#include "Vertex.h"

uint64_t Vertex::packData(int id, glm::ivec3 pos, glm::vec3 normal, const unsigned char texCoords) {
    if(id >= pow(2,14))
        throw std::runtime_error("Vertex ID exceeds maximum value of 2^14 - 1");
    if(pos.x >= pow(2,7) || pos.y >= pow(2,7) || pos.z >= pow(2,7))
        throw std::runtime_error("Vertex position exceeds maximum value of 2^7 - 1");
    if(normal.x < -1.0f || normal.x > 1.0f || normal.y < -1.0f || normal.y > 1.0f || normal.z < -1.0f || normal.z > 1.0f)
        throw std::runtime_error("Vertex normal exceeds range of -1.0 to 1.0");
    uint64_t packed = 0;
    packed |= ((uint64_t)(id) & 0x7FF) << 53; // ID (11 bits)
    packed |= ((uint64_t)(pos.x) & 0x7F) << 46; // Position X (7 bits)
    packed |= ((uint64_t)(pos.y) & 0x7F) << 39; // Position Y (7 bits)
    packed |= ((uint64_t)(pos.z) & 0x7F) << 32; // Position Z (7 bits)
    packed |= ((uint64_t)(std::round((normal.x + 1.0f) * 511.5f)) & 0x3FF) << 22; // Normal X (10 bits)
    packed |= ((uint64_t)(std::round((normal.y + 1.0f) * 511.5f)) & 0x3FF) << 12; // Normal X (10 bits)
    packed |= ((uint64_t)(std::round((normal.z + 1.0f) * 511.5f)) & 0x3FF) << 2; // Normal X (10 bits)
    packed |= (uint64_t)((texCoords >> 1 & 0x1) << 1); // TexCoord X (1 bit)
    packed |= (uint64_t)(texCoords & 0x1); // TexCoord Y (1 bit)
    return packed;
}