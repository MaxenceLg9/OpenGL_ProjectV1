#include "cglm/cglm.h"
#include "glad/glad.h"
#include "GLFW/glfw3.h"

#include <vector>
#include <cmath>
#include <cstdio>

#include "display/callback/callback.h"
#include "display/window/window.h"
#include "display/world/chunk/chunk.h"
#include "display/world/player/player.h"
#include "math/math.h"
#include "display/window/cursor/cursor.h"
#include "display/world/World.h"


WINDOW window;

void framebuffercallback(GLFWwindow *w, const int width, const int height) {
    window.width = width;
    window.height = height;
    glViewport(0, 0, width, height);
}

int main() {
    // Init GLFW
    if (!glfwInit()) return -1;
    glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR, 4);
    glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR, 6);
    glfwWindowHint(GLFW_OPENGL_PROFILE, GLFW_OPENGL_CORE_PROFILE);

    const GLFWvidmode *mode = glfwGetVideoMode(glfwGetPrimaryMonitor());

    window.width = mode->width;
    window.height = mode->height;
    printf("Mode Refresh rate %d\n", mode->refreshRate);

    // Create window
    window.window = glfwCreateWindow(window.width, window.height, "Triangle", NULL, NULL);
    if (!window.window) {
        glfwTerminate();
        return -1;
    }

    glfwSetWindowMonitor(window.window, NULL, 0, 0, window.width, window.height, mode->refreshRate);
    glfwMakeContextCurrent(window.window);
    glfwSetKeyCallback(window.window, key_callback);
    glfwSetInputMode(window.window, GLFW_CURSOR, GLFW_CURSOR_DISABLED);
    glfwSetCursorPosCallback(window.window, mouse_callback);
    glfwSetScrollCallback(window.window, scroll_callback);
    glfwSetFramebufferSizeCallback(window.window, framebuffercallback);

    // Load OpenGL functions
    if (!gladLoadGLLoader((GLADloadproc) glfwGetProcAddress)) {
        fprintf(stderr, "Failed to initialize GLAD\n");
        return -1;
    }


    // Shader shader("assets/shaders/chunk/vertex.ls", "assets/shaders/chunk/fragment.ls");


    glEnable(GL_DEPTH_TEST);
    glPolygonMode(GL_FRONT,GL_FILL);
    glEnable(GL_CULL_FACE);
    glFrontFace(GL_CW); // Counter-clockwise is front
    glCullFace(GL_BACK); // Cull back faces


    // Player *player = new Player(0.0f, 0.0f, 0.0f);
    Player player(0.0f, 0.0f, 0.0f);
    glfwSetWindowUserPointer(window.window, &player);
    World world;
    Chunk chunk;
    Cursor cursor;
    // glfwDestroyWindow(window.window);
    // glfwTerminate();
    // return 0;
    double deltaTime(0.0f), lastFrame(0.0f); // Time of last frame
    while (!glfwWindowShouldClose(window.window)) {
        double currentFrame = glfwGetTime();
        player.setDeltaTime(currentFrame - lastFrame);
        lastFrame = currentFrame;
        glClearColor(0.15f, 0.65f, 1.0f, 1.0f);
        glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);



        glm::vec3 cameraPos(player.getCoords());
        glm::vec3 cameraTarget = cameraPos + player.getDirection();

        // build view matrix
        glm::mat4 view = glm::lookAt(cameraPos, cameraTarget, player.getUp());
        glm::mat4 projection = glm::perspective(glm_rad(player.getFov()), (float) window.width / (float) window.height, 0.01f, 1000.0f);
        glm::mat4 pro_view = projection * view;

        glDepthFunc(GL_LESS);
        world.render(pro_view);
        // chunk.render(shader, pro_view, glm::vec3(0.0f, 0.0f, 0.0f));

        GLenum err;
        while ((err = glGetError()) != GL_NO_ERROR) {
            printf("OpenGL error: %x\n", err);
        }


        cursor.drawCursor(window);
        handleKeysPressed(window.window, &player);
        glfwSwapBuffers(window.window);
        glfwPollEvents();
    }

    printf("End\n");
    glfwSetWindowUserPointer(window.window,nullptr);
    glfwDestroyWindow(window.window);
    glfwTerminate();
    return 0;
}