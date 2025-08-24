//
// Created by Sinis on 31/05/2025.
//

#include "Vertex.h"

uint64_t Vertex::packData(const int id, const glm::ivec3 pos, const int face_id, const unsigned char texCoords) {
    if(id >= pow(2,18))
        throw std::runtime_error("Vertex ID exceeds maximum value of 2^18 - 1");
    if(pos.x >= pow(2,7) || pos.y >= pow(2,7) || pos.z >= pow(2,7))
        throw std::runtime_error("Vertex position exceeds maximum value of 2^7 - 1");
    if(face_id > 5 || face_id < 0)
        throw std::runtime_error("Face id has to be between 0 and 5");
    uint64_t packed = 0;
    packed |= ((uint64_t)(id) & 0x3FFFF) << 46; // ID (18 bits)
    packed |= ((uint64_t)(pos.x) & 0x7F) << 39; // Position X (7 bits)
    packed |= ((uint64_t)(pos.y) & 0x7F) << 32; // Position Y (7 bits)
    packed |= ((uint64_t)(pos.z) & 0x7F) << 25; // Position Z (7 bits)
    packed |= (uint64_t)(face_id & 0x7) << 18; // Face ID (3 bits)
    packed |= (uint64_t)((texCoords >> 1 & 0x1) << 1); // TexCoord X (1 bit)
    packed |= (uint64_t)(texCoords & 0x1); // TexCoord Y (1 bit)
    return packed;
}