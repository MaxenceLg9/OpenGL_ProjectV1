#include "glad/glad.h"
#include "GLFW/glfw3.h"

#include <cstdio>

#include "game/render/callback/callback.h"
#include "game/render/window.h"
#include "game/render/world/chunk/chunk.h"
#include "game/render/gui/cursor/cursor.h"
#include "game/render/world/World.h"
#include "game/render/world/light/light.h"
#include "utils/logs/Logs.h"
#include "utils/OpenGL/OpenGL.h"


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

    // Create window
    window.OGLwindow = glfwCreateWindow(window.width, window.height, "MeinKraft", glfwGetPrimaryMonitor(), NULL);
    if (!window.OGLwindow) {
        glfwTerminate();
        return -1;
    }
    Logs::debug("Initializing OpenGL window with size: " + std::to_string(window.width) + "/" + std::to_string(window.height) + " and framerate " + std::to_string(mode->refreshRate));
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

    check_opengl_error("main1");

    glEnable(GL_DEPTH_TEST);
    glPolygonMode(GL_FRONT_AND_BACK,GL_FILL);
    glEnable(GL_CULL_FACE);
    glFrontFace(GL_CW); // Counter-clockwise is front
    glCullFace(GL_BACK); // Cull back faces
    check_opengl_error("main2");

    const auto world = new World(&window);
    const auto cursor = new Cursor();

    double lastFrame(glfwGetTime()); // Time of last frame
    while (!glfwWindowShouldClose(window.OGLwindow)) {
        const double currentFrame = glfwGetTime();

        world->tick(currentFrame - lastFrame);
        lastFrame = currentFrame;
        glClearColor(0.15f, 0.65f, 1.0f, 1.0f);
        glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);

        world->build_chunk_mesh(); // Build chunks that are ready
        world->render();
        // Checking for OpenGL errors after each loop
        check_opengl_error("loop");
        cursor->drawCursor(window);

        glfwSwapBuffers(window.OGLwindow);
        glfwPollEvents();
    }
    glfwSetWindowUserPointer(window.OGLwindow,nullptr);
    delete cursor;
    delete world;
    glfwDestroyWindow(window.OGLwindow);
    glfwTerminate();
    Logs::close();
    Logs::debug("Ending program");
    return 0;
}