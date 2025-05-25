//
// Created by maxence on 25/05/25.
//

#ifndef WORLD_H
#define WORLD_H
#include <memory>
#include <vector>

#include "chunk/chunk.h"
#include "player/player.h"
#include "../../display/callback/callback.h"


class World {
public:
    World();

    ~World();

    void render(glm::mat4 pro_view) const;

private:
    std::vector<std::unique_ptr<Chunk>> world;
    Shader chunkShader;
};



#endif //WORLD_H
