//
// Created by maxence on 24/05/25.
//

#ifndef LIGHT_H
#define LIGHT_H

#include "../../model/mesh/shader/shader.h"
#include "../../model/mesh/mesh.h"
#include <glm.hpp>

class Light {
public:
    Light();
    ~Light();
    void render(const glm::mat4 & p_v, const glm::vec3 pos) const;
private:
    void build_mesh(std::vector<VERTEX> &vertexdata, std::vector<unsigned int> &indices);

    static int addData(std::vector<VERTEX> &vertex, std::vector<unsigned int> &indices, VERTEX *v, int index);
    Mesh *mesh;
    Shader shader;
};

#endif //LIGHT_H
