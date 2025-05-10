//
// Created by maxence on 03/05/25.
//

#ifndef CALLBACK_H
#define CALLBACK_H

#include "../world/player/player.h"
#include "GLFW/glfw3.h"

extern float mixValue;
extern double angle;
extern float fov;

enum KEY_STATUS{
    PRESSED = 1,
    RELEASED = 0
};

typedef struct  {
    enum KEY_STATUS status;
    void (* function)();
    int fnNumber;
} KEYS;


void mouse_callback(GLFWwindow* window, double xpos, double ypos);

void scroll_callback(GLFWwindow* window, double xoffset, double yoffset);

void key_callback(GLFWwindow* window, int key, int scancode, int action, int mods);

void handleKeysPressed(GLFWwindow *w, Player *player);

#endif //CALLBACK_H
