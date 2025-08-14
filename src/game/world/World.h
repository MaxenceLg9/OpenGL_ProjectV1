//
// Created by maxence on 25/05/25.
//

#ifndef WORLD_H
#define WORLD_H

#define WORLD_SIZE 8
#define WORLD_THREADS 16


#include <memory>
#include <vector>
#include <atomic>

#include "chunk/chunk.h"
#include "../../math/math.h"
#include "player/player.h"
#include "../../display/model/mesh/shader/shader.h"
#include "light/light.h"
#include "../../display/window/window.h"
#include "../../display/callback/callback.h"
#include "../../display/model/mesh/texture/TextureArray.h"

class Chunk;


class World {
public:
    explicit World(WINDOW *window);

    ~World();

    void render() const;

    int getBlockAt(glm::ivec3 ipos) const;

    void generate_chunks(short part);

    void build_chunk_mesh();

    void tick(double deltaTime);

    void addChunkToBuild(const glm::ivec3 &pos, Chunk *chunk);

    void addChunksToBuild(std::map<glm::ivec3, Chunk *, IVec3Compare> *localChunks);

    /**
     * After building the chunk mesh asynchronously, needs to be linked with openGL from main thread
     * Links all chunk meshes that are ready to be linked.
     */
    void link_chunk_meshes();

private:

    void create_chunks();

    std::map<glm::ivec3, Chunk *,IVec3Compare> chunks;
    std::map<glm::ivec3, Chunk *,IVec3Compare> chunksToBuild;
    std::map<glm::ivec3, Chunk *,IVec3Compare> chunksToLink;

    std::atomic<bool> isBuilding = false;

    TextureArray texture;

    Shader chunkShader;
    Light light;
    Player player;
    WINDOW *window;
    mutable std::string logMessage;

    std::mutex buildLock;
    std::mutex linkLock;

    void thread_chunk_mesh();
};



#endif //WORLD_H
