#include <pthread.h>
#include <stdatomic.h>
#include <stdio.h>
#include <stdlib.h>    // for malloc/free
#include <unistd.h>
#include <GLFW/glfw3.h>
#include "special_callback.h"

#include "../world/player.h"

float mixValue = 0.5f;
double angle = 0.0f;

KEYS keys[GLFW_KEY_LAST + 1] = {
    {RELEASED, NULL, 0},
};

void key_callback(GLFWwindow *window, int key, int scancode, int action, int mods) {
    if (action == GLFW_PRESS) {
        printf("Key pressed %d\n",key);
        keys[key].status = PRESSED;
    }
    if (action == GLFW_RELEASE) {
        printf("Key released %d\n",key);
        keys[key].status = RELEASED;
    }
}

void handleKeysPressed(GLFWwindow *w) {
    for (int i = 0; i < GLFW_KEY_LAST + 1; i++) {
        if (keys[i].status == PRESSED) {
            const int key = i;

            if (key == GLFW_KEY_DOWN) {
                if (mixValue > 0.0f) {
                    mixValue -= 0.02f;
                    if (mixValue < 0.0f) mixValue = 0.0f;
                } else {
                    break;
                }
            }

            if (key == GLFW_KEY_UP) {
                if (mixValue < 1.0f) {
                    mixValue += 0.02f;
                    if (mixValue > 1.0f) mixValue = 1.0f;
                }
            }

            if (key == GLFW_KEY_LEFT) {
                angle += 0.01f;
            }

            if (key == GLFW_KEY_RIGHT) {
                angle -= 0.01f;
            }

            if (key == GLFW_KEY_W) {
                addToZ(1.0f);
            }

            if (key == GLFW_KEY_S) {
                addToZ(-1.0f);
            }

            if (key == GLFW_KEY_A) {
                addToX(0.1f);
            }

            if (key == GLFW_KEY_D) {
                addToX(-0.1f);
            }

            if (key == GLFW_KEY_SPACE) {
                addToY(-0.1f);
            }

            if (key == GLFW_KEY_LEFT_CONTROL) {
                addToY(0.1f);
            }

            if (key == GLFW_KEY_Z) {
                addToMouse(-0.1f,0.0f);
            }

            if (key == GLFW_KEY_X) {
                addToMouse(0.1f,0.0f);
            }

            if (key == GLFW_KEY_ESCAPE) {
                glfwSetWindowShouldClose(w, GLFW_TRUE);
            }
        }
    }
}
