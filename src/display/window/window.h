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
    GLFWwindow *GLFWwindow;
} WINDOW;

class Window{
public:
private:
    int width;
    int height;
    const char *title;
    GLFWwindow *GLFWwindow;
};

#endif //WINDOW_H
