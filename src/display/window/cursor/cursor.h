//
// Created by Sinis on 08/05/2025.
//

#ifndef CURSOR_H
#define CURSOR_H
#include "glad/glad.h"
#include "GLFW/glfw3.h"
#include "../window.h"
#include "../../model/mesh/shader/shader.h"
#include "../../model/mesh/mesh.h"


class Cursor {
public:
    void drawCursor(WINDOW w);
    Cursor();
    ~Cursor() {
        printf("Destroying cursor\n");
    }

    static std::vector<VERTEX> vertices();

    static std::vector<unsigned int> indices();

private:
    Shader shader;
    Mesh mesh;
};

#endif //CURSOR_H
