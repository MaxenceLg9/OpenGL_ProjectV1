//
// Created by maxence on 25/05/25.
//

#include "World.h"

#include <glm.hpp>
#include <cglm/util.h>
#include <thread>
#include <ext/matrix_clip_space.hpp>
#include <ext/matrix_transform.hpp>

#include <GLFW/glfw3.h>

#include "player/player.h"
#include "../../logs/Logs.h"

World::World(WINDOW *window) : chunkShader("assets/shaders/chunk/vertex.vert", "assets/shaders/chunk/fragment.frag"), player(1.0f, 0.0f, 1.0f), window(window), logMessage("Creating world\n") {
    glfwSetWindowUserPointer(window->OGLwindow, &player);
    create_chunks();
    Logs::log("INFO", logMessage);
    chunkShader.use();
    chunkShader.setInt("texture1", 0);
    chunkShader.setInt("texture2", 1);
}

void World::create_chunks(){
    printf("Creating World\n");
    time_t t = time(nullptr);
    logMessage.append("World size: " + std::to_string(WORLD_SIZE) + "x" + std::to_string(WORLD_SIZE) + "x" + std::to_string(WORLD_SIZE) + "\n");
    std::mutex lock;
    for (int i = 0; i < WORLD_SIZE; i++) {
        for (int j = 0; j < WORLD_SIZE; j++) {
            for (int k = 0; k < WORLD_SIZE; k++) {
                logMessage.append("Creating chunk at position: " + std::to_string(i) + "," + std::to_string(j) + "," + std::to_string(k) + "\n");
                new Chunk(glm::ivec3(i,j,k),&world, &lock);
//                world.emplace(glm::ivec3(i,j,k),std::make_unique<Chunk>());
            }
        }
    }
    std::this_thread::sleep_for(std::chrono::seconds(2));
    printf("World created in %lld seconds\nCreating Meshs for each chunks\n", time(nullptr) - t);
    for(auto &[pos, chunk] : world) {
        logMessage.append("Chunk at position: " + std::to_string(pos.x) + "," + std::to_string(pos.y) + "," + std::to_string(pos.z) + "\n");
        chunk->build_mesh(*this, pos);
    }
    printf("World & Mesh created in %lld\n", time(nullptr) - t);
}

World::~World(){
    printf("Destroying world\n");
    for (auto &[pos, chunk] : world) {
        delete chunk; // Free the ChunkMesh
    }
    window = nullptr;
}

void World::render() const {

    glm::vec3 cameraPos(player.getCoords());
    glm::vec3 cameraTarget = cameraPos + player.getDirection();

    // build view matrix
    glm::mat4 view = glm::lookAt(cameraPos, cameraTarget, player.getUp());
    glm::mat4 projection = glm::perspective(glm_rad(player.getFov()), (float) window->width / (float) window->height, 0.01f, 1000.0f);
    glm::mat4 pro_view = projection * view;

    glDepthFunc(GL_LESS);
    light.render(pro_view, player.getCoords() + glm::vec3(0.0f, 100.0f, 0.0f));

    chunkShader.use();
    // camera/view transformation
    glm::vec3 color = light.getColor();
    chunkShader.setVec3("color", color.x,color.y,color.z);
    int n = 0;
    for (int i = 0; i < WORLD_SIZE; i++) {
        for (int j = 0; j < WORLD_SIZE; j++) {
            for (int k = 0; k < WORLD_SIZE; k++) {
                if (!world.contains(glm::ivec3(i,j, k)))
                    continue;
                glm::mat4 model(1.0f);
                model = glm::translate(model, glm::vec3(i, j, k) * (float) CHUNK_SIZE);
                chunkShader.setMatrix4fv("p_v_m", glm::value_ptr(pro_view * model));
                world.at(glm::ivec3(i,j, k))->render();
                n++;
            }
        }
    }
    // printf("Rendered %d chunks\n", n);
}

int World::getBlockAt(const glm::ivec3 ipos) const {
    glm::ivec3 chunkPos(ipos.x / CHUNK_SIZE, ipos.y / CHUNK_SIZE, ipos.z / CHUNK_SIZE);
    glm::ivec3 blockPos(ipos.x % CHUNK_SIZE, ipos.y % CHUNK_SIZE, ipos.z % CHUNK_SIZE);

    // Check if the chunk exists
    if (!world.contains(chunkPos)) {
        return 0; // Return 0 for empty space
    }
//    logMessage.append("Block position out of bounds: " + std::to_string(ipos.x) + "," + std::to_string(ipos.y) + "," + std::to_string(ipos.z) + " : " + std::to_string(world.at(chunkPos)->getBlockAt(blockPos)) + "\n");
    // Retrieve the block from the chunk
    return world.at(chunkPos)->getBlockAt(blockPos);
}

void World::tick(const double deltaTime) {
    player.setDeltaTime(deltaTime);
    light.setColor(glfwGetTime());
    handleKeysPressed(window->OGLwindow, &player);
}
