//
// Created by maxence on 8/23/25.
//

#include "OpenGL.h"

#include "../logs/Logs.h"

void check_opengl_error(const std::string &source)
{
    GLenum err;
    while ((err = glGetError()) != GL_NO_ERROR) {
        Logs::debug("OpenGL error at " + source + " " + std::to_string(err));
    }
}
