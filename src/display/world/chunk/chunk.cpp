//
// Created by maxence on 07/05/25.
//

#include "chunk.h"

#include <detail/_noise.hpp>
#include <detail/_noise.hpp>
#include <gtc/bitfield.hpp>
#include <gtc/bitfield.hpp>
#include <gtc/noise.hpp>

#include "../../../math/math.h"
#include "../../callback/callback.h"
#include "../../model/mesh/shader/shader.h"
#include "../../model/mesh/mesh.h"

Chunk::Chunk() {
    blocks[0] = 0;
    for (int x = 0; x < CHUNK_SIZE; x++) {
        for (int y = 0; y < CHUNK_SIZE; y++) {
            for (int z = 0; z < CHUNK_SIZE; z++) {
                if (glm::simplex(glm::vec3(x,y,z) * 0.1f) > 0.2f)
                    blocks[x * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + z] = 1;
                else
                    blocks[x * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + z] = 0;
                blocks[x * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + z] = 1;
            }
        }
    }
    build_mesh(blocks);
    mesh = new Mesh(vertexdata, indices, std::vector<TEXTURE>());
    mesh->loadTextures("assets/textures/blocks/ikrine_ore.png", GL_TEXTURE0, "texture1");
    mesh->loadTextures("assets/textures/blocks/stone.png", GL_TEXTURE1, "texture2");
}

Chunk::~Chunk() {
    printf("Releasing Mesh %p\n",mesh);
    delete mesh;
}

void Chunk::render(const Shader& shader, const glm::mat4 &p_v, const glm::vec3 pos) const {
    glm::mat4 model(1.0f);
    model = glm::translate(model, pos * (float) CHUNK_SIZE);
    // printf("Rendering chunk at %f,%f,%f\n", pos.x, pos.y, pos.z);

    shader.setMatrix4fv("p_v_m", glm::value_ptr(p_v * model));
    // shader.setFloat("mixValue", mixValue);
    mesh->draw(shader);
}


int Chunk::addData(std::vector<VERTEX> &vertex, std::vector<unsigned int> &indices, VERTEX *v, int index) {
    vertex.push_back(v[0]);
    vertex.push_back(v[1]);
    vertex.push_back(v[2]);
    vertex.push_back(v[3]);

    indices.push_back(index);
    indices.push_back(index + 1);
    indices.push_back(index + 2);
    indices.push_back(index);
    indices.push_back(index + 2);
    indices.push_back(index + 3);

    return index + 4;
}

void Chunk::build_mesh(const uint8_t blocks[]) {
    int index = 0;
    for (int x = 0; x < CHUNK_SIZE; x++) {
        for (int y = 0; y < CHUNK_SIZE; y++) {
            for (int z = 0; z < CHUNK_SIZE; z++) {
                uint8_t voxel_id = blocks[x * CHUNK_SIZE * CHUNK_SIZE + y * CHUNK_SIZE + z];
                // uint8_t v0[5],v1[5],v2[5],v3[5];
                if (voxel_id == 0) continue; // skip empty blocks
                VERTEX v[4];
                //front face
                if (isVoid(glm::vec3(x, y, z + 1), blocks)) {
                    v[0].Position = glm::vec3(x, y, z + 1);
                    v[1].Position = glm::vec3(x, y + 1, z + 1);
                    v[2].Position = glm::vec3(x + 1, y + 1, z + 1);
                    v[3].Position = glm::vec3(x + 1, y, z + 1);

                    v[0].TexCoords = glm::vec2(0.0f, 0.0f);
                    v[1].TexCoords = glm::vec2(0.0f, 1.0f);
                    v[2].TexCoords = glm::vec2(1.0f, 1.0f);
                    v[3].TexCoords = glm::vec2(1.0f, 0.0f);

                    index = addData(vertexdata, indices, v, index);


                    /*pack_data(v0, x, y, z + 1, voxel_id, 5);
                    pack_data(v1, x, y + 1, z + 1, voxel_id,5);
                    pack_data(v2, x + 1, y + 1, z + 1, voxel_id,5);
                    pack_data(v3, x + 1, y, z + 1, voxel_id, 5);*/

                    // index = add_data(vertexdata, index, v1, v0, v3, v1, v3, v2);
                }
                // back face
                if (isVoid(glm::vec3(x, y, z - 1), blocks)) {
                    v[0].Position = glm::vec3(x, y, z);
                    v[1].Position = glm::vec3(x + 1, y, z);
                    v[2].Position = glm::vec3(x + 1, y + 1, z);
                    v[3].Position = glm::vec3(x, y + 1, z);

                    v[0].TexCoords = glm::vec2(1.0f, 0.0f);
                    v[1].TexCoords = glm::vec2(0.0f, 0.0f);
                    v[2].TexCoords = glm::vec2(0.0f, 1.0f);
                    v[3].TexCoords = glm::vec2(1.0f, 1.0f);

                    index = addData(vertexdata, indices, v, index);
                    /*pack_data(v0, x, y, z, voxel_id, 4);
                    pack_data(v1, x, y + 1, z, voxel_id,4);
                    pack_data(v2, x + 1, y + 1, z, voxel_id,4);
                    pack_data(v3, x + 1, y, z, voxel_id, 4);*/

                    // index = add_data(vertexdata, index, v1, v0, v3, v1, v3, v2);
                }
                //top face
                if (isVoid(glm::vec3(x, y + 1, z), blocks)) {
                    // format: x, y, z, voxel_id, face_id, ao_id,
                    v[0].Position = glm::vec3(x, y + 1, z);
                    v[1].Position = glm::vec3(x + 1, y + 1, z);
                    v[2].Position = glm::vec3(x + 1, y + 1, z + 1);
                    v[3].Position = glm::vec3(x, y + 1, z + 1);


                    v[0].TexCoords = glm::vec2(1.0f, 0.0f);
                    v[1].TexCoords = glm::vec2(0.0f, 0.0f);
                    v[2].TexCoords = glm::vec2(0.0f, 1.0f);
                    v[3].TexCoords = glm::vec2(1.0f, 1.0f);

                    index = addData(vertexdata, indices, v, index);
                    /*pack_data(v0, x, y + 1, z, voxel_id, 0);
                    pack_data(v1, x + 1, y + 1, z, voxel_id, 0);
                    pack_data(v2, x + 1, y + 1, z + 1, voxel_id, 0);
                    pack_data(v3, x, y + 1, z + 1, voxel_id, 0);*/

                    // index = add_data(vertexdata, index, v1, v0, v3, v1, v3, v2);
                }
                // bottom face
                if (isVoid(glm::vec3(x, y - 1, z), blocks)) {
                    v[0].Position = glm::vec3(x, y, z);
                    v[3].Position = glm::vec3(x + 1, y, z);
                    v[2].Position = glm::vec3(x + 1, y, z + 1);
                    v[1].Position = glm::vec3(x, y, z + 1);

                    v[0].TexCoords = glm::vec2(0.0f, 1.0f);
                    v[3].TexCoords = glm::vec2(0.0f, 0.0f);
                    v[2].TexCoords = glm::vec2(1.0f, 0.0f);
                    v[1].TexCoords = glm::vec2(1.0f, 1.0f);

                    index = addData(vertexdata, indices, v, index);
                    /*pack_data(v0, x, y, z, voxel_id, 1);
                    pack_data(v1, x + 1, y, z, voxel_id, 1);
                    pack_data(v2, x + 1, y, z + 1, voxel_id, 1);
                    pack_data(v3, x, y, z + 1, voxel_id, 1);*/

                    // index = add_data(vertexdata, index, v1, v0, v3, v1, v3, v2);
                }

                // right face
                if (isVoid(glm::vec3(x + 1, y, z), blocks)) {
                    v[0].Position = glm::vec3(x + 1, y, z);
                    v[3].Position = glm::vec3(x + 1, y + 1, z);
                    v[2].Position = glm::vec3(x + 1, y + 1, z + 1);
                    v[1].Position = glm::vec3(x + 1, y, z + 1);

                    v[0].TexCoords = glm::vec2(1.0f, 0.0f);
                    v[3].TexCoords = glm::vec2(1.0f, 1.0f);
                    v[2].TexCoords = glm::vec2(0.0f, 1.0f);
                    v[1].TexCoords = glm::vec2(0.0f, 0.0f);

                    index = addData(vertexdata, indices, v, index);
                    /*pack_data(v0, x + 1, y, z, voxel_id, 2);
                    pack_data(v1, x + 1, y + 1, z, voxel_id, 2);
                    pack_data(v2, x + 1, y + 1, z + 1, voxel_id, 2);
                    pack_data(v3, x + 1, y, z + 1, voxel_id, 2);*/

                    // index = add_data(vertexdata, index, v1, v0, v3, v1, v3, v2);
                }

                // left face
                if (isVoid(glm::vec3(x - 1, y, z), blocks)) {
                    v[0].Position = glm::vec3(x, y, z);
                    v[1].Position = glm::vec3(x, y + 1, z);
                    v[2].Position = glm::vec3(x, y + 1, z + 1);
                    v[3].Position = glm::vec3(x, y, z + 1);

                    v[0].TexCoords = glm::vec2(0.0f, 0.0f);
                    v[1].TexCoords = glm::vec2(0.0f, 1.0f);
                    v[2].TexCoords = glm::vec2(1.0f, 1.0f);
                    v[3].TexCoords = glm::vec2(1.0f, 0.0f);

                    index = addData(vertexdata, indices, v, index);
                    /*pack_data(v0, x, y, z, voxel_id, 3);
                    pack_data(v1, x, y + 1, z, voxel_id, 3);
                    pack_data(v2, x, y + 1, z + 1, voxel_id,3);
                    pack_data(v3, x, y, z + 1, voxel_id, 3);*/

                    // index = add_data(vertexdata, index, v1, v0, v3, v1, v3, v2);
                }
            }
        }
    }
}

bool Chunk::isVoid(glm::vec3 pos, const uint8_t blocks[]) {
    if (pos.x < 0 || pos.x >= CHUNK_SIZE ||
        pos.y < 0 || pos.y >= CHUNK_SIZE ||
        pos.z < 0 || pos.z >= CHUNK_SIZE) {
        return true;
    }
    return blocks[(int) pos.x * CHUNK_SIZE * CHUNK_SIZE + (int) pos.y * CHUNK_SIZE + (int) pos.z] == 0;
}
