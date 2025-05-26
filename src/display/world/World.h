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
    World();

    ~World();

    void render(glm::mat4 pro_view) const;

    int getBlockAt(glm::vec3 pos) const;

private:
    std::map<glm::ivec3,std::unique_ptr<Chunk>,IVec3Compare> world;
    Shader chunkShader;
};



#endif //WORLD_H
