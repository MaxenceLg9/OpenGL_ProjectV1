//
// Created by Sinis on 08/05/2025.
//

#ifndef CURSOR_H
#define CURSOR_H

#include "../../model/mesh/shader/shader.h"
#include "../../model/mesh/mesh.h"

class Cursor {
public:
    void drawCursor();
    Cursor();
private:
    Shader shader;
    Mesh mesh;
};

#endif //CURSOR_H
