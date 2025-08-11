//
// Created by maxence on 25/05/25.
//

#ifndef WORLD_H
#define WORLD_H

#define WORLD_SIZE 6
#define WORLD_THREADS 8


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

private:
    std::map<glm::ivec3, Chunk *,IVec3Compare> chunks;
    std::map<glm::ivec3, Chunk *,IVec3Compare> chunksToBuild;

    Texture texture;

    Shader chunkShader;
    Light light;
    Player player;
    WINDOW *window;
    mutable std::string logMessage;
    std::mutex lock;

    void create_chunks();
};



#endif //WORLD_H
