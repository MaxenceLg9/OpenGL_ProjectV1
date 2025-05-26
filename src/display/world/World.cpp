//
// Created by maxence on 25/05/25.
//

#include "World.h"
#include "World.h"
#include "World.h"

#include <glm.hpp>
#include <cglm/util.h>
#include <ext/matrix_clip_space.hpp>
#include <ext/matrix_transform.hpp>

#include <GLFW/glfw3.h>

#include "player/player.h"

World::World() : chunkShader("assets/shaders/chunk/vertex.ls", "assets/shaders/chunk/fragment.ls") {
    for (int i = 0; i < 10; i++) {
        world.emplace(glm::ivec3(0,0,i),std::make_unique<Chunk>(*this));
    }
    chunkShader.use();
    chunkShader.setInt("texture1", 0);
    chunkShader.setInt("texture2", 1);
}

World::~World(){
    printf("Destroying world\n");
}

void World::render(const glm::mat4 pro_view) const {
    chunkShader.use();
    // camera/view transformation

    float t = (float) sin(glfwGetTime() * 4) * 0.25 + 0.75;
    chunkShader.setVec3("color", t, t, t);
    for (int i = 0; i < 1; i++) {
        for (int j = 0; j < 10; j++) {
            world.at(glm::ivec3(i,0, j))->render(chunkShader, pro_view, glm::vec3(i, 0, j));
        }
    }
}

int World::getBlockAt(glm::vec3 pos) const {
    if (!world.contains(glm::ivec3(pos.x / 64,pos.y / 64, + pos.z / 64)))
        return 0;
    return world.at(glm::ivec3(pos.x / 64,pos.y / 64, + pos.z / 64))->getBlockAt(glm::ivec3((int) pos.x % 64, (int) pos.y, (int) pos.z % 64));
}
