//
// Created by maxence on 25/05/25.
//

#include "World.h"

#include "glm.hpp"
#include <ranges>
#include "cglm/util.h"
#include <thread>
#include "ext/matrix_clip_space.hpp"
#include "ext/matrix_transform.hpp"

#include "GLFW/glfw3.h"

#include "player/player.h"
#include "../../../utils/logs/Logs.h"
#include "block/block.h"

World::World(WINDOW* window) : texture("textures"),
                               chunkShader("assets/shaders/chunk/vertex.vert", "assets/shaders/chunk/fragment.frag"), player(-1.0f, 0.0f, -1.0f), window(window),
                               logMessage("Creating world\n")
{
    //    texture.addTexture("assets/textures/blocks/ikrine_block.png",10);
    //    texture.addTexture("assets/textures/blocks/ikrine_ore.png",11);
    texture.addTexture("assets/textures/blocks/stone.png", BlockType::STONE - 1);
    texture.addTexture("assets/textures/blocks/dirt.png", BlockType::DIRT - 1);
    texture.addTexture("assets/textures/blocks/deepslate.png", BlockType::DEEPSLATE - 1);
    //    texture.addTexture("assets/textures/blocks/platinum_ore.png",12);
    //    texture.addTexture("assets/textures/blocks/stone_bricks.png",20);
    glfwSetWindowUserPointer(window->OGLwindow, &player);
    create_chunks();
    Logs::log("INFO", logMessage);
    chunkShader.use();
}

void World::create_chunks()
{
    printf("Creating World\n");
    time_t t = time(nullptr);
    logMessage.append(
        "World size: " + std::to_string(WORLD_SIZE) + "x" + std::to_string(WORLD_SIZE) + "x" +
        std::to_string(WORLD_SIZE) + "\n");
    Logs::debug("Render distance : " + std::to_string(WORLD_SIZE) + " chunks");
    for (short i = 0; i < WORLD_THREADS; i++)
    {
        std::thread(&World::generate_chunks, this, i).detach();
    }
}

void World::generate_chunks(short part)
{
    time_t t = time(nullptr);
    int totalChunks = WORLD_SIZE * WORLD_SIZE * WORLD_SIZE;
    int chunkPerThread = (totalChunks + WORLD_THREADS - 1) / WORLD_THREADS;
    std::map<glm::ivec3, Chunk*, IVec3Compare> localChunks;

    int startIndex = part * chunkPerThread;
    int endIndex = startIndex + chunkPerThread > totalChunks ? totalChunks : startIndex + chunkPerThread;
    for (int index = startIndex; index < endIndex; index++)
    {
        int i = index / (WORLD_SIZE * WORLD_SIZE);
        int j = (index / WORLD_SIZE) % WORLD_SIZE;
        int k = index % WORLD_SIZE;

        //        Logs::debug("Thread " + std::to_string(part) + " generating chunk at position: " + std::to_string(i) + "," + std::to_string(j) + "," + std::to_string(k));
        localChunks.emplace(glm::ivec3(i, j, k), new Chunk(glm::ivec3(i, j, k), this));
    }
    this->chunksLock.lock();
    for (auto& [pos,chunk] : localChunks)
    {
        chunks.emplace(pos,chunk);
    }
    chunksLock.unlock();
    Logs::debug(
        "Thread " + std::to_string(part + 1) + " finished generating chunks in " + std::to_string(time(nullptr) - t) +
        " seconds\n" +
        " generated from " + std::to_string(startIndex) + " to " + std::to_string(endIndex) + " chunks");
}


void World::build_chunk_mesh()
{
    if (chunks.empty())
        return;
    if (isBuilding)
        return;
    isBuilding = true;
    std::thread([this]()
    {
        thread_chunk_mesh();
        isBuilding = false;
    }).detach();
}

void World::thread_chunk_mesh()
{
    std::vector<glm::ivec3> toErase;
    //    Logs::debug(std::to_string(chunksToBuild.size()) + " chunks to build");

    for (auto& [pos, chunk] : chunks)
    {
        //        logMessage.append("Building mesh for chunk at position: " + std::to_string(pos.x) + "," + std::to_string(pos.y) + "," + std::to_string(pos.z) + "\n");
        glm::ivec3 xNeg(pos.x - 1, pos.y, pos.z);
        glm::ivec3 xPos(pos.x + 1, pos.y, pos.z);
        glm::ivec3 yNeg(pos.x, pos.y - 1, pos.z);
        glm::ivec3 yPos(pos.x, pos.y + 1, pos.z);
        glm::ivec3 zNeg(pos.x, pos.y, pos.z - 1);
        glm::ivec3 zPos(pos.x, pos.y, pos.z + 1);

        if (
            (chunks.contains(xNeg) || pos.x == 0) &&
            (chunks.contains(xPos) || pos.x == WORLD_SIZE - 1) &&
            (chunks.contains(yNeg) || pos.y == 0) &&
            (chunks.contains(yPos) || pos.y == WORLD_SIZE - 1) &&
            (chunks.contains(zNeg) || pos.z == 0) &&
            (chunks.contains(zPos) || pos.z == WORLD_SIZE - 1)
        )
        {
            //            Logs::debug("Adding the chunk in the vector");
            meshesLock.lock();
            meshes.emplace(pos,chunk->build_mesh());
            meshesLock.unlock();
            //            Logs::debug("Pushing the pos value to remove the chunk from chunksToBuild after the loop");
            toErase.push_back(pos);
            //            Logs::debug("Releasing the buildLock");
            break;
        }
        // Logs::debug("Skipping chunk mesh building for the chunk at pos : " + std::to_string(pos.x) + "," + std::to_string(pos.y) + "," + std::to_string(pos.z) + "\n");
    }
    //    Logs::debug("Locking the mutex to remove the chunk from chunksToBuild");
    if (toErase.empty())
        return;
    //    Logs::debug("Finished building chunk meshes");
}

World::~World()
{
    Logs::debug("Destroying world and releasing chunks");
    for (const auto& chunk : chunks | std::views::values)
    {
        delete chunk; // Free the ChunkMesh
    }
    for (const auto& mesh : meshes | std::views::values)
    {
        delete mesh; // Free the ChunkMesh
    }
    window = nullptr;
    Logs::debug("World destroyed");
}

void World::render() const
{
    //    Logs::debug("Rendering world");
    glm::vec3 cameraPos(player.getCoords());
    glm::vec3 cameraTarget = cameraPos + player.getDirection();

    // build view matrix
    glm::mat4 view = glm::lookAt(cameraPos, cameraTarget, player.getUp());
    glm::mat4 projection = glm::perspective(glm_rad(player.getFov()), (float)window->width / (float)window->height,
                                            0.01f, 1000.0f);
    glm::mat4 pro_view = projection * view;

    glDepthFunc(GL_LESS);
    // light.render(pro_view, player.getCoords() + glm::vec3(0.0f, 100.0f, 0.0f));

    chunkShader.use();
    texture.use_textures(chunkShader);
    // camera/view transformation
    //    glm::vec3 color = light.getColor();
    glm::vec3 color = glm::vec3(1.0f, 1.0f, 1.0f); // Default color for debugging
    chunkShader.setVec3("color", color.x, color.y, color.z);
    int n = 0;
    for (int i = 0; i < WORLD_SIZE; i++)
    {
        for (int j = 0; j < WORLD_SIZE; j++)
        {
            for (int k = 0; k < WORLD_SIZE; k++)
            {
                if (!meshes.contains(glm::ivec3(i, j, k)))
                    continue;
                ChunkMesh *mesh = meshes.at(glm::ivec3(i, j, k));
                if (!mesh->is_linked())
                    mesh->link();
                glm::mat4 model(1.0f);
                model = glm::translate(model, glm::vec3(i, j, k) * (float)CHUNK_SIZE);
                chunkShader.setMatrix4fv("p_v_m", glm::value_ptr(pro_view * model));
                mesh->draw();
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
    if (!chunks.contains(chunkPos))
    {
        return 0; // Return 0 for empty space
    }
    //    logMessage.append("Block position out of bounds: " + std::to_string(ipos.x) + "," + std::to_string(ipos.y) + "," + std::to_string(ipos.z) + " : " + std::to_string(world.at(chunkPos)->getBlockAt(blockPos)) + "\n");
    return chunks.at(chunkPos)->getBlockAt(blockPos);
}

void World::tick(const double deltaTime)
{
    player.setDeltaTime(deltaTime);
    //    light.setColor(glfwGetTime());
    // light.setColor(100);
    handleKeysPressed(window->OGLwindow, &player);
}
