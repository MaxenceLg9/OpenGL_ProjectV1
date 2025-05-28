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
#include "display/world/light/light.h"


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
    window.OGLwindow = glfwCreateWindow(window.width, window.height, "Triangle", NULL, NULL);
    if (!window.OGLwindow) {
        glfwTerminate();
        return -1;
    }

    glfwSetWindowMonitor(window.OGLwindow, NULL, 0, 0, window.width, window.height, mode->refreshRate);
    glfwMakeContextCurrent(window.OGLwindow);
    glfwSetKeyCallback(window.OGLwindow, key_callback);
    glfwSetInputMode(window.OGLwindow, GLFW_CURSOR, GLFW_CURSOR_DISABLED);
    glfwSetCursorPosCallback(window.OGLwindow, mouse_callback);
    glfwSetScrollCallback(window.OGLwindow, scroll_callback);
    glfwSetFramebufferSizeCallback(window.OGLwindow, framebuffercallback);

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

    World world(&window);
    Cursor cursor;
    // glfwDestroyWindow(window.window);
    // glfwTerminate();
    // return 0;
    double deltaTime(0.0f), lastFrame(0.0f); // Time of last frame
    while (!glfwWindowShouldClose(window.OGLwindow)) {
        double currentFrame = glfwGetTime();

        world.tick(currentFrame - lastFrame);
        lastFrame = currentFrame;
        glClearColor(0.15f, 0.65f, 1.0f, 1.0f);
        glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

        glDepthFunc(GL_LESS);

        world.render();
        GLenum err;
        while ((err = glGetError()) != GL_NO_ERROR) {
            printf("OpenGL error: %x\n", err);
        }
        cursor.drawCursor(window);

        glfwSwapBuffers(window.OGLwindow);
        glfwPollEvents();
    }

    printf("End\n");
    glfwSetWindowUserPointer(window.OGLwindow,nullptr);
    glfwDestroyWindow(window.OGLwindow);
    glfwTerminate();
    return 0;
}