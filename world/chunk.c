//
// Created by maxence on 07/05/25.
//

#include "chunk.h"

#include "../libs/special_callback.h"
#include "../libs/shader.h"

void renderChunk(const CHUNK *chunk, const GLint n, const SHADER *shader) {
    for (int i = 0; i < 64; i++) {
        for (int j = 0; j < 64; j++) {
            for (int k = 0; k < 64; k++) {
                // render each block in the chunk

                float pos[3] = {chunk->position[0] + i, chunk->position[1] + j, chunk->position[2] + k};
                mat4 model;
                vec3 rotateAxis = {0.3f, 0.6f, 0.9f};

                glm_mat4_identity(model);
                glm_translate(model, pos);
                glm_rotate(model, (float) angle, rotateAxis);


                setMatrix4fv(shader,"model",model[0]);
                setFloat(shader, "mixValue", mixValue);
                glDrawElements(GL_TRIANGLES, n, GL_UNSIGNED_INT, 0);
            }
        }
    }

}
