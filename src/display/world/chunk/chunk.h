//
// Created by maxence on 07/05/25.
//

#ifndef CHUNK_H
#define CHUNK_H

# define CHUNK_SIZE 16

#include "glm.hpp"
#include "../../model/mesh/shader/shader.h"
#include "../block/block.h"
#include "../../model/mesh/mesh.h"

typedef struct {
    glm::vec3 position;
    Block cubes[CHUNK_SIZE][CHUNK_SIZE][CHUNK_SIZE]; // 64x64x64 cubes
} CHUNK;

void renderChunk(const CHUNK *chunk, Mesh mesh, Shader shader);

#endif //CHUNK_H
