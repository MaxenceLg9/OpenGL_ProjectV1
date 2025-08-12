//
// Created by maxence on 25/05/25.
//

#ifndef WORLD_H
#define WORLD_H

#define WORLD_SIZE 5
#define WORLD_THREADS 16


#include <memory>
#include <vector>

#include "chunk/chunk.h"
#include "../../math/math.h"
#include "player/player.h"
#include "../model/mesh/shader/shader.h"
#include "light/light.h"
#include "../../display/window/window.h"
#include "../../display/callback/callback.h"

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
private:

    void create_chunks();

    std::map<glm::ivec3, Chunk *,IVec3Compare> chunks;
    std::map<glm::ivec3, Chunk *,IVec3Compare> chunksToBuild;

    Texture texture;

    Shader chunkShader;
    Light light;
    Player player;
    WINDOW *window;
    mutable std::string logMessage;
    std::mutex lock;


};



#endif //WORLD_H
