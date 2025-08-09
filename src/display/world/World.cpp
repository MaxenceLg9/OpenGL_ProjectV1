//
// Created by maxence on 25/05/25.
//

#include "World.h"

#include <glm.hpp>
#include <ranges>
#include <cglm/util.h>
#include <thread>
#include <ext/matrix_clip_space.hpp>
#include <ext/matrix_transform.hpp>

#include <GLFW/glfw3.h>

#include "player/player.h"
#include "../../logs/Logs.h"

World::World(WINDOW* window) : chunkShader("assets/shaders/chunk/vertex.vert", "assets/shaders/chunk/fragment.frag"),
                               player(1.0f, 0.0f, 1.0f), window(window), logMessage("Creating world\n")
{
    glfwSetWindowUserPointer(window->OGLwindow, &player);
    create_chunks();
    Logs::log("INFO", logMessage);
    chunkShader.use();
    chunkShader.setInt("texture1", 0);
    chunkShader.setInt("texture2", 1);
}

void World::create_chunks()
{
    printf("Creating World\n");
    time_t t = time(nullptr);
    logMessage.append(
        "World size: " + std::to_string(WORLD_SIZE) + "x" + std::to_string(WORLD_SIZE) + "x" +
        std::to_string(WORLD_SIZE) + "\n");
    Logs::debug("Render distance : " + std::to_string(WORLD_SIZE) + " chunks");
    for (int i = 0; i < WORLD_SIZE; i++)
    {
        for (int j = 0; j < WORLD_SIZE; j++)
        {
            for (int k = 0; k < WORLD_SIZE; k++)
            {
                logMessage.append("Creating chunk at position: " + std::to_string(i) + "," + std::to_string(j) + "," + std::to_string(k) + "\n");
                new Chunk(glm::ivec3(i, j, k), &chunksToBuild, &lock);
                //                world.emplace(glm::ivec3(i,j,k),std::make_unique<Chunk>());
            }
        }
    }
}

void World::build_chunk_mesh()
{
    std::vector<glm::ivec3> toErase;
    Logs::debug(std::to_string(chunksToBuild.size()) + " chunks to build");

    for (auto& [pos, chunk] : chunksToBuild) {
        logMessage.append("Building mesh for chunk at position: " + std::to_string(pos.x) + "," + std::to_string(pos.y) + "," + std::to_string(pos.z) + "\n");
        glm::ivec3 xNeg(pos.x - 1, pos.y, pos.z);
        glm::ivec3 xPos(pos.x + 1, pos.y, pos.z);
        glm::ivec3 yNeg(pos.x, pos.y - 1, pos.z);
        glm::ivec3 yPos(pos.x, pos.y + 1, pos.z);
        glm::ivec3 zNeg(pos.x, pos.y, pos.z - 1);
        glm::ivec3 zPos(pos.x, pos.y, pos.z + 1);

        if (
            (chunksToBuild.contains(xNeg) || chunks.contains(xNeg) || pos.x == 0) &&
            (chunksToBuild.contains(xPos) || chunks.contains(xPos) || pos.x == WORLD_SIZE - 1) &&
            (chunksToBuild.contains(yNeg) || chunks.contains(yNeg) || pos.y == 0) &&
            (chunksToBuild.contains(yPos) || chunks.contains(yPos) || pos.y == WORLD_SIZE - 1) &&
            (chunksToBuild.contains(zNeg) || chunks.contains(zNeg) || pos.z == 0) &&
            (chunksToBuild.contains(zPos) || chunks.contains(zPos) || pos.z == WORLD_SIZE - 1)
        ) {
            Logs::debug(
                "Building mesh for chunk at position: " + std::to_string(pos.x) + "," + std::to_string(pos.y) + "," +
                std::to_string(pos.z));
            chunk->build_mesh(*this, pos);
            Logs::debug("Adding the chunk in the vector");
            chunks.emplace(pos, chunk);
            Logs::debug("Pushing the pos value to remove the chunk from chunksToBuild after the loop");
            toErase.push_back(pos);
            Logs::debug("Releasing the lock");
            break;
        }
        // Logs::debug("Skipping chunk mesh building for the chunk at pos : " + std::to_string(pos.x) + "," + std::to_string(pos.y) + "," + std::to_string(pos.z) + "\n");
    }
    Logs::debug("Locking the mutex to remove the chunk from chunksToBuild");
    if (toErase.empty())
        return;
    lock.lock();
    for (auto& pos : toErase)
    {
        chunksToBuild.erase(pos);
    }
    lock.unlock();
    Logs::debug("Finished building chunk meshes");
}

World::~World()
{
    printf("Destroying world\n");
    for (const auto& chunk : chunks | std::views::values)
    {
        delete chunk; // Free the ChunkMesh
    }
    window = nullptr;
}

void World::render() const
{
    Logs::debug("Rendering world");
    glm::vec3 cameraPos(player.getCoords());
    glm::vec3 cameraTarget = cameraPos + player.getDirection();

    // build view matrix
    glm::mat4 view = glm::lookAt(cameraPos, cameraTarget, player.getUp());
    glm::mat4 projection = glm::perspective(glm_rad(player.getFov()), (float)window->width / (float)window->height,
                                            0.01f, 1000.0f);
    glm::mat4 pro_view = projection * view;

    glDepthFunc(GL_LESS);
    light.render(pro_view, player.getCoords() + glm::vec3(0.0f, 100.0f, 0.0f));

    chunkShader.use();
    // camera/view transformation
    glm::vec3 color = light.getColor();
    chunkShader.setVec3("color", color.x, color.y, color.z);
    int n = 0;
    for (int i = 0; i < WORLD_SIZE; i++)
    {
        for (int j = 0; j < WORLD_SIZE; j++)
        {
            for (int k = 0; k < WORLD_SIZE; k++)
            {
                if (!chunks.contains(glm::ivec3(i, j, k)))
                    continue;
                glm::mat4 model(1.0f);
                model = glm::translate(model, glm::vec3(i, j, k) * (float)CHUNK_SIZE);
                chunkShader.setMatrix4fv("p_v_m", glm::value_ptr(pro_view * model));
                chunks.at(glm::ivec3(i, j, k))->render();
                n++;
            }
        }
    }
    // printf("Rendered %d chunks\n", n);
}

int World::getBlockAt(const glm::ivec3 ipos) const
{
    glm::ivec3 chunkPos(ipos.x / CHUNK_SIZE, ipos.y / CHUNK_SIZE, ipos.z / CHUNK_SIZE);
    glm::ivec3 blockPos(ipos.x % CHUNK_SIZE, ipos.y % CHUNK_SIZE, ipos.z % CHUNK_SIZE);

    // Check if the chunk exists
    if (!chunks.contains(chunkPos) && !chunksToBuild.contains(chunkPos))
    {
        return 0; // Return 0 for empty space
    }
    //    logMessage.append("Block position out of bounds: " + std::to_string(ipos.x) + "," + std::to_string(ipos.y) + "," + std::to_string(ipos.z) + " : " + std::to_string(world.at(chunkPos)->getBlockAt(blockPos)) + "\n");
    if (chunks.contains(chunkPos))
        return chunks.at(chunkPos)->getBlockAt(blockPos);
    return chunksToBuild.at(chunkPos)->getBlockAt(blockPos);
}

void World::tick(const double deltaTime)
{
    player.setDeltaTime(deltaTime);
    light.setColor(glfwGetTime());
    handleKeysPressed(window->OGLwindow, &player);
}
