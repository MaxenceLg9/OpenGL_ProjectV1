//
// Created by Sinis on 31/05/2025.
//

#ifndef VERTEX_H
#define VERTEX_H

#include <cstdint>
#include <glm.hpp>
#include <cmath>
#include <stdexcept>

namespace Vertex {
    uint64_t packData(int id, glm::ivec3 pos, glm::vec3 normal, unsigned char texCoords);
} // namespace Vertex

#endif //VERTEX_H
