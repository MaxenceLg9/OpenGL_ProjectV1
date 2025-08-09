#include "glad/glad.h"
#include "GLFW/glfw3.h"

#include <cstdio>

#include "display/callback/callback.h"
#include "display/window/window.h"
#include "display/world/chunk/chunk.h"
#include "display/window/cursor/cursor.h"
#include "display/world/World.h"
#include "display/world/light/light.h"
#include "logs/Logs.h"


WINDOW window;

void framebuffercallback(GLFWwindow *w, const int width, const int height) {
    window.width = width;
    window.height = height;
    glViewport(0, 0, width, height);
}

int main() {
    Logs::init();
    // Init GLFW
    if (!glfwInit()) return -1;
    glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR, 4);
    glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR, 6);
    glfwInitHint(GLFW_PLATFORM, GLFW_PLATFORM_X11);
    glfwWindowHint(GLFW_OPENGL_PROFILE, GLFW_OPENGL_CORE_PROFILE);

    Logs::log("INFO", "Initializing GLFW");

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

    Logs::log("INFO", "Window created successfully");

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


    // Shader shader("assets/shaders/chunk/vertex.vert", "assets/shaders/chunk/fragment.frag");


    glEnable(GL_DEPTH_TEST);
    glPolygonMode(GL_FRONT,GL_FILL);
    glEnable(GL_CULL_FACE);
    glFrontFace(GL_CW); // Counter-clockwise is front
    glCullFace(GL_BACK); // Cull back faces

    World world(&window);
    auto *cursor = new Cursor();

    double lastFrame(glfwGetTime()); // Time of last frame
    while (!glfwWindowShouldClose(window.OGLwindow)) {
        double currentFrame = glfwGetTime();

        world.tick(currentFrame - lastFrame);
        lastFrame = currentFrame;
        glClearColor(0.15f, 0.65f, 1.0f, 1.0f);
        glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

        world.build_chunk_mesh(); // Build chunks that are ready
        world.render();
        GLenum err;
        while ((err = glGetError()) != GL_NO_ERROR) {
            printf("OpenGL error: %x\n", err);
        }
        cursor->drawCursor(window);

        glfwSwapBuffers(window.OGLwindow);
        glfwPollEvents();
    }

    printf("End\n");
    glfwSetWindowUserPointer(window.OGLwindow,nullptr);
    glfwDestroyWindow(window.OGLwindow);
    glfwTerminate();
    delete cursor;
    Logs::close();
    return 0;
}