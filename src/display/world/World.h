//
// Created by maxence on 25/05/25.
//

#ifndef WORLD_H
#define WORLD_H
#include <memory>
#include <vector>

#include "chunk/chunk.h"
#include "player/player.h"
#include "../model/mesh/shader/shader.h"
#include "light/light.h"
#include "../../display/window/window.h"
#include "../../display/callback/callback.h"

class Chunk;

struct IVec3Compare {
    bool operator()(const glm::ivec3& a, const glm::ivec3& b) const {
        if (a.x != b.x) return a.x < b.x;
        if (a.y != b.y) return a.y < b.y;
        return a.z < b.z;
    }
};


class World {
public:
    explicit World(WINDOW *window);

    ~World();

    void render() const;

    int getBlockAt(glm::ivec3 ipos) const;

    void tick(double deltaTime);

private:
    std::map<glm::ivec3,std::unique_ptr<Chunk>,IVec3Compare> world;
    Shader chunkShader;
    Light light;
    Player player;
    WINDOW *window;
    mutable std::string logMessage;
};



#endif //WORLD_H
