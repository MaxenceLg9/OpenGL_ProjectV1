//
// Created by maxence on 24/05/25.
//

#ifndef LIGHT_H
#define LIGHT_H

#define SIZE 1000

#include "../../../render/mesh/shader/shader.h"
#include "../../../render/mesh/vertex/Vertex.h"
#include "../../../render/mesh/mesh.h"
#include <vector>
#include "glm.hpp"

class Light {
public:
    Light();
    ~Light();
    void render(const glm::mat4 & p_v, glm::vec3 playerPos) const;

    glm::vec3 getColor() const;
    void setColor(glm::vec3 color);

private:
    static void build_mesh(std::vector<VERTEX> &vertexdata, std::vector<unsigned int> &indices);

    static int addData(std::vector<VERTEX> &vertex, std::vector<unsigned int> &indices, VERTEX *v, int index);

    Mesh *mesh;
    glm::vec3 color;
    Shader shader;
};

#endif //LIGHT_H
