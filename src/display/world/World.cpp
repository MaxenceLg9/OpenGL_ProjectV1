//
// Created by maxence on 25/05/25.
//

#include "World.h"

#include <glm.hpp>
#include <cglm/util.h>
#include <ext/matrix_clip_space.hpp>
#include <ext/matrix_transform.hpp>

#include <GLFW/glfw3.h>
#include <gtc/type_ptr.inl>

#include "player/player.h"

World::World(WINDOW *window) : chunkShader("assets/shaders/chunk/vertex.ls", "assets/shaders/chunk/fragment.ls"), player(1.0f, 0.0f, 1.0f), window(window) {
    glfwSetWindowUserPointer(window->OGLwindow, &player);
    for (int i = 0; i < 1; i++) {
        for (int j = 0; j < 10; j++) {
            for (int k = 0; k < 10; k++) {
                world.emplace(glm::ivec3(0,0,k),std::make_unique<Chunk>(*this, glm::ivec3(0,0,k)));
            }
        }
    }
    printf("Block %d\n", this->getBlockAt(glm::ivec3(43,52,320)));
    chunkShader.use();
    chunkShader.setInt("texture1", 0);
    chunkShader.setInt("texture2", 1);
}

World::~World(){
    printf("Destroying world\n");
    window = NULL;
}

void World::render() const {

    glm::vec3 cameraPos(player.getCoords());
    glm::vec3 cameraTarget = cameraPos + player.getDirection();

    // build view matrix
    glm::mat4 view = glm::lookAt(cameraPos, cameraTarget, player.getUp());
    glm::mat4 projection = glm::perspective(glm_rad(player.getFov()), (float) window->width / (float) window->height, 0.01f, 1000.0f);
    glm::mat4 pro_view = projection * view;

    light.render(pro_view, player.getCoords() + glm::vec3(0.0f, 100.0f, 0.0f));

    chunkShader.use();
    // camera/view transformation
    glm::vec3 color = light.getColor();
    chunkShader.setVec3("color", color.x,color.y,color.z);
    int n = 0;
    for (int i = 0; i < 1; i++) {
        for (int j = 0; j < 10; j++) {
            for (int k = 0; k < 10; k++) {
                if (!world.contains(glm::ivec3(i,j, k)))
                    continue;
                // printf("Rendering chunk at %d,%d,%d\n", i, 0, j);
                world.at(glm::ivec3(i,j, k))->render(chunkShader, pro_view, glm::vec3(i, j, k));
                n++;
            }
        }
    }
    // printf("Rendered %d chunks\n", n);
}

int World::getBlockAt(const glm::ivec3 ipos) const {
    glm::ivec3 chunkPos(ipos.x / CHUNK_SIZE, ipos.y / CHUNK_SIZE, ipos.z / CHUNK_SIZE);
    glm::ivec3 blockPos(ipos.x % CHUNK_SIZE, ipos.y % CHUNK_SIZE, ipos.z % CHUNK_SIZE);

    // Handle negative modulo results
    if (blockPos.x < 0) blockPos.x += CHUNK_SIZE;
    if (blockPos.y < 0) blockPos.y += CHUNK_SIZE;
    if (blockPos.z < 0) blockPos.z += CHUNK_SIZE;

    // Check if the chunk exists
    if (!world.contains(chunkPos)) {
        return 0; // Return 0 for empty space
    }
    if (blockPos.z == 1 && world.at(chunkPos)->getBlockAt(blockPos) == 0) {
        printf("Warning: Block position %d,%d,%d is out of bounds in chunk %d,%d,%d\n", ipos.x, ipos.y, ipos.z, chunkPos.x, chunkPos.y, chunkPos.z);
    }
    // Retrieve the block from the chunk
    return world.at(chunkPos)->getBlockAt(blockPos);
}

void World::tick(const double deltaTime) {
    player.setDeltaTime(deltaTime);
    light.setColor(glfwGetTime());
    handleKeysPressed(window->OGLwindow, &player);
}
