//
// Created by Sinis on 08/05/2025.
//

#include "cursor.h"
#include "../../model/mesh/shader/shader.h"

float vertices[] = {
        //coords                //texture coords
        //front 0
        -0.1f, 0.5f, 0.0f,
        0.1f, 0.5f, 0.0f,
        0.1f, -0.5f, 0.0f,
        -0.1f, -0.5f, 0.0f,

        //back 4
        0.5f, 0.1f, -0.0f,
        -0.5f, 0.1f, -0.0f,
        0.5f, -0.1f, -0.0f,
        -0.5f, -0.1f, -0.0f
};

unsigned int indices[] = {
        // note that we start from 0!
        //front
        0, 1, 2, // first triangle
        0, 3, 2, // second triangle
        //back
        4, 6, 7, // third triangle
        4, 5, 7, // fourth triangle
};



std::vector<TEXTURE> texturesArray;

std::vector<VERTEX> verticesFromArray(){
    std::vector<VERTEX> verticesArray;
    for(int i = 0; i < sizeof(vertices)/sizeof(vertices[0]); i += 3) {
        VERTEX vertex;
        vertex.Position[0] = vertices[i];
        vertex.Position[1] = vertices[i + 1];
        vertex.Position[2] = vertices[i + 2];
        vertex.TexCoords[0] = 0.0f;
        vertex.TexCoords[1] = 0.0f;
        vertex.Normal[0] = 1.0f;
        vertex.Normal[1] = 1.0f;
        vertex.Normal[2] = 1.0f;
        verticesArray.push_back(vertex);
    }
    return verticesArray;
}

std::vector<unsigned int> indicesFromArray(){
    std::vector<unsigned int> indicesArray;
    for(unsigned int indice : indices) {
        indicesArray.push_back(indice);
    }
    return indicesArray;
}


Cursor::Cursor() : shader("assets/shaders/cursor/vertex.ls", "assets/shaders/cursor/fragment.ls"),
                   mesh(verticesFromArray(),indicesFromArray(),texturesArray)
                   {
}

void Cursor::drawCursor(){
    glDepthFunc(GL_ALWAYS); // Always pass the depth test (same effect as glDisable(GL_DEPTH_TEST))
    mesh.draw(shader);
}
