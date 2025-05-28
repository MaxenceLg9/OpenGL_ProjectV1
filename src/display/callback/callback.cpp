#include <pthread.h>
#include <unistd.h>
#include "GLFW/glfw3.h"
#include <cstdio>
#include "callback.h"

#include "../../math/math.h"

#include "../world/player/player.h"
#include "cglm/cglm.h"

float mixValue = 0.5f;
double angle = 0.0f;

KEYS keys[GLFW_KEY_LAST + 1] = {
    {RELEASED},
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
void scroll_callback(GLFWwindow* window, double xoffset, double yoffset)
{
    Player *player = (Player *) glfwGetWindowUserPointer(window);
    player->addFov((float)yoffset*10);
}

void mouse_callback(GLFWwindow* window, double xpos, double ypos) {
    static float lastX = 960.0f;
    static float lastY = 540.0f;
    static int firstMouse = 1;

    Player *player = (Player *) glfwGetWindowUserPointer(window);

    if (firstMouse) {
        lastX = (float)xpos;
        lastY = (float)ypos;
        firstMouse = 0;
    }

    float xoffset = (float)xpos - lastX;
    float yoffset = lastY - (float)ypos;
    lastX = (float)xpos;
    lastY = (float)ypos;

    float sensitivity = 0.12f;  // much smaller for fine rotation
    xoffset *= -sensitivity;
    yoffset *= -sensitivity;

    player->moveCamera(xoffset, yoffset);
}



void handleKeysPressed(GLFWwindow *w, Player *player) {
    for (int i = 0; i <= GLFW_KEY_LAST; i++) {
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
                player->moveForward(1.f);
            }
            if (key == GLFW_KEY_S) {
                player->moveForward(-1.f);
            }
            if (key == GLFW_KEY_D) {
                player->moveRight(1.f);
            }
            if (key == GLFW_KEY_A) {
                player->moveRight(-1.f);
            }

            if (key == GLFW_KEY_SPACE) {
                player->moveUp(1.f);
            }

            if (key == GLFW_KEY_LEFT_CONTROL) {
                player->moveUp(-1.f);
            }
            if (key == GLFW_KEY_Z) {
                player->makeRoll(-1.0f);  // roll left
            }
            if (key == GLFW_KEY_X) {
                player->makeRoll(1.0f);  // roll right
            }

            if(key == GLFW_KEY_LEFT_SHIFT){
                player->addSpeedMultiplier(GLFW_KEY_LEFT_SHIFT,100);
            }

            if (key == GLFW_KEY_ESCAPE) {
                glfwSetWindowShouldClose(w, GLFW_TRUE);
            }
        }
        if (keys[i].status == RELEASED) {
            if (i == GLFW_KEY_LEFT_SHIFT) {
                player->removeSpeedMultiplier(GLFW_KEY_LEFT_SHIFT);
            }
        }
    }
}
