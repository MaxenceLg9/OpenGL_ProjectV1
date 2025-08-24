//
// Created by maxence on 24/05/25.
//

#include "light.h"

#include "../World.h"
#include "../../../../math/math.h"
#include "../../../../utils/logs/Logs.h"

Light::Light() : color(1.0f, 1.0f, 1.0f), shader("assets/shaders/light/vertex.vert", "assets/shaders/light/fragment.frag") {
    Logs::debug("Creating light");
    std::vector<VERTEX> vertices;
    std::vector<unsigned int> indices;
    build_mesh(vertices,indices);
    mesh = new Mesh(vertices, indices, std::vector<TEXTURE>());
}

glm::vec3 Light::getColor() const {
    return color;
}

Light::~Light() {
    Logs::debug("Releasing Mesh");
    delete mesh;
}

void Light::render(const glm::mat4 &p_v, const glm::vec3 playerPos) const {
    glm::mat4 model(1.0f);
    model = glm::translate(model, playerPos);
    // printf("Rendering chunk at %f,%f,%f\n", pos.x, pos.y, pos.z);
    shader.use();

    shader.setMatrix4fv("p_v_m", glm::value_ptr(p_v * model));
    shader.setVec3("color", color.x, color.y, color.z); // Set light color to white
    mesh->draw(shader);
}


int Light::addData(std::vector<VERTEX> &vertex, std::vector<unsigned int> &indices, VERTEX *v, int index) {
    vertex.push_back(v[0]);
    vertex.push_back(v[1]);
    vertex.push_back(v[2]);
    vertex.push_back(v[3]);

    indices.push_back(index);
    indices.push_back(index + 1);
    indices.push_back(index + 2);
    indices.push_back(index);
    indices.push_back(index + 2);
    indices.push_back(index + 3);

    return index + 4;
}

void Light::setColor(const glm::vec3 color) {
    this->color = color;
}

void Light::build_mesh(std::vector<VERTEX> &vertexdata, std::vector<unsigned int> &indices) {
    int index = 0;
    int x = 0, y = 0, z = 0;
    VERTEX v[4];
    //front face
    v[0].Position = glm::vec3(x, y, z + SIZE);
    v[1].Position = glm::vec3(x, y + SIZE, z + SIZE);
    v[2].Position = glm::vec3(x + SIZE, y + SIZE, z + SIZE);
    v[3].Position = glm::vec3(x + SIZE, y, z + SIZE);

    index = addData(vertexdata, indices, v, index);

    // back face
    v[0].Position = glm::vec3(x, y, z);
    v[1].Position = glm::vec3(x + SIZE, y, z);
    v[2].Position = glm::vec3(x + SIZE, y + SIZE, z);
    v[3].Position = glm::vec3(x, y + SIZE, z);

    index = addData(vertexdata, indices, v, index);
    //top face
    // format: x, y, z, voxel_id, face_id, ao_id,
    v[0].Position = glm::vec3(x, y + SIZE, z);
    v[1].Position = glm::vec3(x + SIZE, y + SIZE, z);
    v[2].Position = glm::vec3(x + SIZE, y + SIZE, z + SIZE);
    v[3].Position = glm::vec3(x, y + SIZE, z + SIZE);

    index = addData(vertexdata, indices, v, index);

    // bottom face
    v[0].Position = glm::vec3(x, y, z);
    v[3].Position = glm::vec3(x + SIZE, y, z);
    v[2].Position = glm::vec3(x + SIZE, y, z + SIZE);
    v[1].Position = glm::vec3(x, y, z + SIZE);

    index = addData(vertexdata, indices, v, index);

    // right face

    v[0].Position = glm::vec3(x + SIZE, y, z);
    v[3].Position = glm::vec3(x + SIZE, y + SIZE, z);
    v[2].Position = glm::vec3(x + SIZE, y + SIZE, z + SIZE);
    v[1].Position = glm::vec3(x + SIZE, y, z + SIZE);

    index = addData(vertexdata, indices, v, index);


    // left face
    v[0].Position = glm::vec3(x, y, z);
    v[1].Position = glm::vec3(x, y + SIZE, z);
    v[2].Position = glm::vec3(x, y + SIZE, z + SIZE);
    v[3].Position = glm::vec3(x, y, z + SIZE);

    addData(vertexdata, indices, v, index);

    for (auto &vertex : vertexdata) {
        Logs::log("INFO", "Vertex position: " + std::to_string(vertex.Position.x) + ", " + std::to_string(vertex.Position.y) + ", " + std::to_string(vertex.Position.z));
    }
}
