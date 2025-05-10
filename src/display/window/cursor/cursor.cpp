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

void drawCursor(){
    static SHADER cursorShader;
    getShader(&cursorShader, "shaders/cursor/vertex.ls", "shaders/cursor/fragment.ls");
}
