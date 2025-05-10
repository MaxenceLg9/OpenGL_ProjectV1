//
// Created by maxence on 07/05/25.
//

#include "chunk.h"
#include "../../../math/math.h"
#include "../../callback/callback.h"
#include "../../model/mesh/shader/shader.h"
#include "../../model/mesh/mesh.h"

void renderChunk(const CHUNK *chunk,Mesh mesh,Shader shader) {
    float chunkPos[3] = {chunk->position[0]*CHUNK_SIZE, chunk->position[1]*CHUNK_SIZE, chunk->position[2]*CHUNK_SIZE};
    for (int i = 0; i < CHUNK_SIZE; i++) {
        for (int j = 0; j < 1; j++) {
            for (int k = 0; k < CHUNK_SIZE; k++) {
                // render each block in the chunk

                glm::vec3 pos(chunkPos[0] + i, chunkPos[1] + j, chunkPos[2] - k);
                glm::mat4 model(1.0f);
                glm::vec3 rotateAxis = {0.3f, 0.6f, 0.9f};
                model = glm::translate(model, pos);
                model = glm::rotate(model, (float) angle, rotateAxis);


                shader.setMatrix4fv("model",glm::value_ptr(model));
                shader.setFloat("mixValue", mixValue);
                mesh.draw(shader);
            }
        }
    }
}