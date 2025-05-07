//
// Created by maxence on 07/05/25.
//

#ifndef CHUNK_H
#define CHUNK_H

#include <cglm/cglm.h>
#include "../libs/shader.h"
#include "block.h"

typedef struct {
    vec3 position;
    BLOCK cubes[16][16][16]; // 64x64x64 cubes
} CHUNK;

void renderChunk(const CHUNK *chunk, GLint n, const SHADER *shader);

#endif //CHUNK_H
