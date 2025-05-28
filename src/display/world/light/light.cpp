//
// Created by maxence on 24/05/25.
//

#include "light.h"

#include "../World.h"
#include "../../../math/math.h"
#include "../../callback/callback.h"
#include "../../model/mesh/shader/shader.h"
#include "../../model/mesh/mesh.h"

Light::Light() : color(1.0f, 1.0f, 1.0f), shader("assets/shaders/light/vertex.ls", "assets/shaders/light/fragment.ls") {
    std::vector<VERTEX> vertices;
    std::vector<unsigned int> indices;
    build_mesh(vertices,indices);
    mesh = new Mesh(vertices, indices, std::vector<TEXTURE>());
}

glm::vec3 Light::getColor() const {
    return color;
}

Light::~Light() {
    printf("Releasing Mesh %p\n", mesh);
    delete mesh;
}

void Light::render(const glm::mat4 &p_v, const glm::vec3 playerPos) const {
    glm::mat4 model(1.0f);
    model = glm::translate(model, playerPos);
    // printf("Rendering chunk at %f,%f,%f\n", pos.x, pos.y, pos.z);
    shader.use();

    shader.setMatrix4fv("p_v_m", glm::value_ptr(p_v * model));
    shader.setVec3("color", 1.0f, 1.0f, 1.0f); // Set light color to white
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

void Light::setColor(double deltaTime) {
    float speed = 2.0f; // Speed of oscillation
    color.r = (sin(deltaTime * speed) + 1.0f) / 2.0f; // Red oscillates between 0 and 1
    color.g = (sin(deltaTime * speed + glm::pi<float>() / 2) + 1.0f) / 2.0f; // Green offset by 90 degrees
    color.b = (sin(deltaTime * speed + glm::pi<float>()) + 1.0f) / 2.0f; // Blue offset by 180 degrees
}

void Light::build_mesh(std::vector<VERTEX> &vertexdata, std::vector<unsigned int> &indices) {
    int index = 0;
    int x = 0, y = 0, z = 0;
    VERTEX v[4];
    //front face
    v[0].Position = glm::vec3(x, y, z + 10);
    v[1].Position = glm::vec3(x, y + 10, z + 10);
    v[2].Position = glm::vec3(x + 10, y + 10, z + 10);
    v[3].Position = glm::vec3(x + 10, y, z + 10);

    index = addData(vertexdata, indices, v, index);

    // back face
    v[0].Position = glm::vec3(x, y, z);
    v[1].Position = glm::vec3(x + 10, y, z);
    v[2].Position = glm::vec3(x + 10, y + 10, z);
    v[3].Position = glm::vec3(x, y + 10, z);

    index = addData(vertexdata, indices, v, index);
    //top face
    // format: x, y, z, voxel_id, face_id, ao_id,
    v[0].Position = glm::vec3(x, y + 10, z);
    v[1].Position = glm::vec3(x + 10, y + 10, z);
    v[2].Position = glm::vec3(x + 10, y + 10, z + 10);
    v[3].Position = glm::vec3(x, y + 10, z + 10);

    index = addData(vertexdata, indices, v, index);

    // bottom face
    v[0].Position = glm::vec3(x, y, z);
    v[3].Position = glm::vec3(x + 10, y, z);
    v[2].Position = glm::vec3(x + 10, y, z + 10);
    v[1].Position = glm::vec3(x, y, z + 10);

    index = addData(vertexdata, indices, v, index);

    // right face

    v[0].Position = glm::vec3(x + 10, y, z);
    v[3].Position = glm::vec3(x + 10, y + 10, z);
    v[2].Position = glm::vec3(x + 10, y + 10, z + 10);
    v[1].Position = glm::vec3(x + 10, y, z + 10);

    index = addData(vertexdata, indices, v, index);


    // left face
    v[0].Position = glm::vec3(x, y, z);
    v[1].Position = glm::vec3(x, y + 10, z);
    v[2].Position = glm::vec3(x, y + 10, z + 10);
    v[3].Position = glm::vec3(x, y, z + 10);

    addData(vertexdata, indices, v, index);

    for (auto &vertex : vertexdata) {
        printf("Vertex: %f, %f, %f\n", vertex.Position.x, vertex.Position.y, vertex.Position.z);
    }
}
