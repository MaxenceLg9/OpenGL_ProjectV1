#include "cglm/cglm.h"
#include "GLAD/glad.h"
#include "GLFW/glfw3.h"

#include <vector>
#include <cmath>
#include <cstdio>

#include "display/callback/callback.h"
#include "display/window/window.h"
#include "display/world/chunk/chunk.h"
#include "display/world/player/player.h"
#include "display/model/mesh/mesh.h"
#include "math/math.h"
#include "display/window/cursor/cursor.h"


WINDOW window;

void framebuffercallback(GLFWwindow *w, const int width, const int height) {
    window.width = width;
    window.height = height;
    glViewport(0, 0, width, height);
}

// Cube vertices (positions only)
float verticesArray[] = {
        //coords                //texture coords
        //front 0
        -0.5f, 0.5f, 0.5f,      1.0f, 1.0f,// top left front
        0.5f, 0.5f, 0.5f,       0.0f, 1.0f,// top right front
        0.5f, -0.5f, 0.5f,      0.0f, 0.0f,// bottom right front
        -0.5f, -0.5f, 0.5f,     1.0f, 0.0f,// bottom left front

        //back 4
        0.5f, 0.5f, -0.5f,      1.0f, 1.0f,// top right back
        -0.5f, 0.5f, -0.5f,     0.0f, 1.0f,// top left back
        0.5f, -0.5f, -0.5f,     1.0f, 0.0f,// bottom right back
        -0.5f, -0.5f, -0.5f,    0.0f, 0.0f,// bottom left back

        //left 8
        -0.5f, -0.5f, 0.5f,     0.0f, 0.0f,// bottom left front
        -0.5f, -0.5f, -0.5f,    1.0f, 0.0f,// bottom left back
        -0.5f, 0.5f, 0.5f,      0.0f, 1.0f,// top left front
        -0.5f, 0.5f, -0.5f,     1.0f, 1.0f,// top left back

        //right 12
        0.5f, 0.5f, 0.5f,       1.0f, 1.0f,// top right front
        0.5f, 0.5f, -0.5f,      0.0f, 1.0f,// top right back
        0.5f, -0.5f, 0.5f,      1.0f, 0.0f,// bottom right front
        0.5f, -0.5f, -0.5f,     0.0f, 0.0f,// bottom right back

        //top 16
        0.5f, 0.5f, -0.5f,      1.0f, 0.0f,// top right back
        0.5f, 0.5f, 0.5f,       1.0f, 1.0f,// top right front
        -0.5f, 0.5f, 0.5f,      0.0f, 1.0f,// top left front
        -0.5f, 0.5f, -0.5f,     0.0f, 0.0f,// top left back

        //bottom 20
        0.5f, -0.5f, 0.5f,      1.0f, 0.0f,// bottom right front
        0.5f, -0.5f, -0.5f,     1.0f, 1.0f,// bottom right back
        -0.5f, -0.5f, -0.5f,    0.0f, 1.0f,// bottom left back
        -0.5f, -0.5f, 0.5f,     0.0f, 0.0f,// bottom left front

};

unsigned int indicesArray[] = {
        // note that we start from 0!
        //front
        0, 1, 2, // first triangle
        2, 3, 0, // second triangle
        //back
        7, 6, 4, // third triangle
        4, 5, 7, // fourth triangle
        //left
        8, 9, 11, // fifth triangle
        11, 10, 8, // sixth triangle
        //right
        12, 13, 15, // seventh triangle
        15, 14, 12, // eigth triangle
        //top
        16, 17, 18, // ninth triangle
        18, 19, 16, // ten triangle
        //bottom
        20, 21, 22, // eleven triangle
        22, 23, 20, // twelve triangle
};

int main() {
    // Init GLFW
    if (!glfwInit()) return -1;
    glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR, 4);
    glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR, 6);
    glfwWindowHint(GLFW_OPENGL_PROFILE, GLFW_OPENGL_CORE_PROFILE);

    const GLFWvidmode *mode = glfwGetVideoMode(glfwGetPrimaryMonitor());

    window.width = mode->width;
    window.height = mode->height;
    printf("Mode Refresh rate %d\n",mode->refreshRate);

    // Create window
    window.GLFWwindow = glfwCreateWindow(window.width, window.height, "Triangle", NULL, NULL);
    if (!window.GLFWwindow) {
        glfwTerminate();
        return -1;
    }

    glfwSetWindowMonitor(window.GLFWwindow, NULL, 0, 0, window.width, window.height, mode->refreshRate);
    glfwMakeContextCurrent(window.GLFWwindow);
    glfwSetKeyCallback(window.GLFWwindow, key_callback);
    glfwSetInputMode(window.GLFWwindow, GLFW_CURSOR, GLFW_CURSOR_DISABLED);
    glfwSetCursorPosCallback(window.GLFWwindow, mouse_callback);
    glfwSetScrollCallback(window.GLFWwindow, scroll_callback);
    glfwSetFramebufferSizeCallback(window.GLFWwindow, framebuffercallback);

    // Load OpenGL functions
    if (!gladLoadGLLoader((GLADloadproc) glfwGetProcAddress)) {
        fprintf(stderr, "Failed to initialize GLAD\n");
        return -1;
    }

    std::vector<VERTEX> vertices;
    for(int i = 0; i < sizeof(verticesArray) / sizeof(float); i += 5) {
        VERTEX vertex;
        vertex.Position[0] = verticesArray[i];
        vertex.Position[1] = verticesArray[i + 1];
        vertex.Position[2] = verticesArray[i + 2];
        vertex.TexCoords[0] = verticesArray[i + 3];
        vertex.TexCoords[1] = verticesArray[i + 4];
        vertex.Normal[0] = 0.5f;
        vertex.Normal[1] = 0.5f;
        vertex.Normal[2] = 0.5f;
        vertices.push_back(vertex);
    }
    std::vector<unsigned int> indices;
    for(unsigned int i : indicesArray) {
        indices.push_back(i);
    }
    std::vector<TEXTURE> textures;

    Mesh mesh(vertices, indices, textures);

    mesh.loadTextures("assets/textures/blocks/ikrine_ore.png",GL_TEXTURE0,"texture1");
    mesh.loadTextures("assets/textures/blocks/ikrine_block.png",GL_TEXTURE1,"texture2");


    Shader shader("assets/shaders/cube/vertex.ls", "assets/shaders/cube/fragment.ls");

    glEnable(GL_DEPTH_TEST);
    glPolygonMode(GL_FRONT,GL_FILL);
    glEnable(GL_CULL_FACE);
    glCullFace(GL_FRONT);

    glFrontFace(GL_CCW);
    shader.use();
    shader.setInt("texture1", 0);
    shader.setInt("texture2", 1);

    Player player(0.0f, 0.0f, 0.0f);
    glfwSetWindowUserPointer(window.GLFWwindow, &player);
    CHUNK chunk[10][10];
    for (int i = 0; i < 10; i++) {
        for (int j = 0; j < 10; j++) {
            chunk[i][j].position = glm::vec3(i, 0.0f, -j);
        }
    }
    int changed = 0;
    Cursor cursor;
    double deltaTime = 0.0f,lastFrame = 0.0f; // Time of last frame
    while (!glfwWindowShouldClose(window.GLFWwindow)) {
        double currentFrame = glfwGetTime();
        player.setDeltaTime(currentFrame - lastFrame);
        lastFrame = currentFrame;
        glClearColor(0.15f, 0.65f, 1.0f, 1.0f);
        glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);


        shader.use();

        // camera/view transformation
        glm::mat4 view,rollMatrix(1.0f);
        glm::vec3 cameraPos(player.getCoords());
        glm::vec3 cameraTarget, rotatedUp;
        glm::vec3 baseUp = {0.0f, 1.0f, 0.0f};
        // cameraTarget = cameraPos + direction
        cameraTarget = cameraPos + player.getDirection();

        // rotatedUp = rotate(baseUp, roll, around direction)
        // build view matrix
        view = glm::lookAt(cameraPos, cameraTarget, player.getUp());

        shader.setMatrix4fv("view",glm::value_ptr(view));

        glm::mat4 projection;

        projection = glm::perspective(glm_rad(fov), (float) window.width / (float) window.height, 0.01f, 1000.0f);
        shader.setMatrix4fv("projection",glm::value_ptr(projection));

//        mesh.draw(shader);

        glDepthFunc(GL_LESS);
        for (auto & i : chunk) {
//            for (auto & j : i) {
                renderChunk(&i[0],mesh,shader);
//            }
        }
        cursor.drawCursor();
        handleKeysPressed(window.GLFWwindow,&player);
        glfwSwapBuffers(window.GLFWwindow);
        glfwPollEvents();
    }

    shader.freeShader();
    mesh.freeMesh();

    glfwDestroyWindow(window.GLFWwindow);
    glfwTerminate();
    return 0;
}
