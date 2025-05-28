//
// Created by maxence on 04/05/25.
//

#ifndef WINDOW_H
#define WINDOW_H

#include "GLFW/glfw3.h"

typedef struct {
    int width;
    int height;
    const char *title;
    GLFWwindow *OGLwindow;
} WINDOW;

class Window{
public:
    Window();
private:
    int width;
    int height;
    const char *title;
    GLFWwindow *window;
};

#endif //WINDOW_H
