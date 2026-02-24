//
// Created by Sinis on 31/05/2025.
//

#ifndef VERTEX_H
#define VERTEX_H

#include <cstdint>
#include <glm.hpp>
#include <cmath>
#include <stdexcept>

typedef struct {
    glm::vec3 Position;
    glm::vec3 Normal;
    glm::ivec2 TexCoords;
} VERTEX;

namespace Vertex {
    uint64_t packData(int id, glm::ivec3 pos, glm::vec3 lighting, int face_id, unsigned char texCoords);
    uint64_t packData(int id, glm::ivec3 pos, int face_id, unsigned char texCoords);
} // namespace Vertex

#endif //VERTEX_H