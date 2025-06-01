//
// Created by Sinis on 08/05/2025.
//

#include "cursor.h"


#include "../window.h"
#include "../../model/mesh/shader/shader.h"
#include <ext.hpp>



Cursor::Cursor() : shader("assets/shaders/cursor/vertex.vert", "assets/shaders/cursor/fragment.frag"),
                   mesh(Cursor::vertices(),indices(),std::vector<TEXTURE>()) {
}

std::vector<VERTEX> Cursor::vertices() {
    std::vector<VERTEX> vertices;
    //vertical
    vertices.push_back(VERTEX(glm::vec3(-0.003f, 0.02f, 0.0f), glm::vec3(1.0f, 1.0f, 1.0f), glm::ivec2(0.0f, 1.0f)));
    vertices.push_back(VERTEX(glm::vec3(0.003f, 0.02f, 0.0f), glm::vec3(1.0f, 1.0f, 1.0f), glm::ivec2(1.0f, 1.0f)));
    vertices.push_back(VERTEX(glm::vec3(0.003f, -0.02f, 0.0f), glm::vec3(0.0f, 0.0f, 0.0f), glm::ivec2(1.0f, 0.0f)));
    vertices.push_back(VERTEX(glm::vec3(-0.003f, -0.02f, 0.0f), glm::vec3(0.0f, 0.0f, 0.0f), glm::ivec2(0.0f, 0.0f)));

    //horizontal
    vertices.push_back(VERTEX(glm::vec3(-0.02f, 0.003f, -0.0f), glm::vec3(0.0f, 0.0f, 0.0f), glm::ivec2(0.0f, 1.0f)));
    vertices.push_back(VERTEX(glm::vec3(0.02f, 0.003f, -0.0f), glm::vec3(1.0f, 1.0f, 1.0f), glm::ivec2(1.0f, 1.0f)));
    vertices.push_back(VERTEX(glm::vec3(0.02f, -0.003f, -0.0f), glm::vec3(1.0f, 1.0f, 1.0f), glm::ivec2(1.0f, 0.0f)));
    vertices.push_back(VERTEX(glm::vec3(-0.02f, -0.003f, -0.0f), glm::vec3(0.0f, 0.0f, 0.0f), glm::ivec2(0.0f, 0.0f)));

    return vertices;
}

std::vector<unsigned int> Cursor::indices() {
    std::vector<unsigned int> indices;

    //vertical
    indices.push_back(0);
    indices.push_back(1);
    indices.push_back(2);
    indices.push_back(0);
    indices.push_back(2);
    indices.push_back(3);

    //horizontal
    indices.push_back(4);
    indices.push_back(5);
    indices.push_back(6);
    indices.push_back(4);
    indices.push_back(6);
    indices.push_back(7);

    return indices;
}


void Cursor::drawCursor(WINDOW w){
    this->shader.use();
    float aspect = (float)w.width / (float)w.height;
    this->shader.setMatrix4fv("projection",glm::value_ptr(glm::ortho(-1.0f, 1.0f, -1.0f / aspect, 1.0f / aspect, -1.0f, 1.0f)));
    glDepthFunc(GL_ALWAYS); // Always pass the depth test (same effect as glDisable(GL_DEPTH_TEST))
    mesh.draw(shader);
}
