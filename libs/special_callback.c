#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>    // for malloc/free
#include <unistd.h>
#include <GLFW/glfw3.h>
#include "special_callback.h"

#include <cglm/cglm.h>

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
float fov = 90.0f;
void scroll_callback(GLFWwindow* window, double xoffset, double yoffset)
{
    fov -= (float)yoffset*10;
    if (fov < 1.0f)
        fov = 1.0f;
    if (fov > 140.0f)
        fov = 140.0f;
}

void mouse_callback(GLFWwindow* window, double xpos, double ypos) {
    static float lastX = 960.0f;
    static float lastY = 540.0f;
    static int firstMouse = 1;

    PLAYER *player = glfwGetWindowUserPointer(window);

    if (firstMouse) {
        lastX = (float)xpos;
        lastY = (float)ypos;
        firstMouse = 0;
    }

    float xoffset = (float)xpos - lastX;
    float yoffset = lastY - (float)ypos;
    lastX = (float)xpos;
    lastY = (float)ypos;

    float sensitivity = 0.0012f;  // much smaller for fine rotation
    xoffset *= -sensitivity;
    yoffset *= -sensitivity;

    // Create rotation matrices
    mat4 yawMat, pitchMat;
    glm_mat4_identity(yawMat);
    glm_mat4_identity(pitchMat);

    // Rotate around local up (roll-aware) for yaw
    vec3 up;
    vec3 baseUp = {0.0f, 1.0f, 0.0f};
    mat4 rollMat;
    glm_mat4_identity(rollMat);
    glm_rotate(rollMat, glm_rad(player->roll), player->direction);
    glm_mat4_mulv3(rollMat, baseUp, 0.0f, up);  // up vector now roll-aware

    glm_rotate(yawMat, xoffset, up);

    // Rotate around local right vector for pitch
    vec3 right;
    glm_vec3_cross(up, player->direction, right);
    glm_normalize(right);
    glm_rotate(pitchMat, yoffset, right);

    // Apply both rotations to direction
    vec3 newDir;
    mat4 combinedRot;
    glm_mat4_identity(combinedRot);
    glm_mat4_mul(yawMat, pitchMat, combinedRot);
    glm_mat4_mulv3(combinedRot, player->direction, 0.0f, newDir);
    glm_normalize_to(newDir, player->direction);
}



void handleKeysPressed(GLFWwindow *w, PLAYER *player) {
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
                moveForward(0.1f, player);
            }
            if (key == GLFW_KEY_S) {
                moveForward(-0.1f, player);
            }
            if (key == GLFW_KEY_D) {
                moveRight(0.1f, player);
            }
            if (key == GLFW_KEY_A) {
                moveRight(-0.1f, player);
            }

            if (key == GLFW_KEY_SPACE) {
                addToY(0.1f,player);
            }

            if (key == GLFW_KEY_LEFT_CONTROL) {
                addToY(-0.1f,player);
            }
            if (key == GLFW_KEY_Z) {
                player->roll -= 1.0f;  // roll left
            }
            if (key == GLFW_KEY_X) {
                player->roll += 1.0f;  // roll right
            }

            if (key == GLFW_KEY_ESCAPE) {
                glfwSetWindowShouldClose(w, GLFW_TRUE);
            }
        }
    }
}
