#include <glad/glad.h>
#include <GLFW/glfw3.h>
#include <math.h>
#include <stdio.h>
#include "libs/shader.h"
#include "libs/textures.h"
#include "libs/special_callback.h"
#include <cglm/cglm.h>

#include "world/player.h"


// Cube vertices (positions only)
float vertices[] = {
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

unsigned int indices[] = {
    // note that we start from 0!
    //front
    0, 1, 2, // first triangle
    0, 3, 2, // second triangle
    //back
    4, 6, 7, // third triangle
    4, 5, 7, // fourth triangle
    //left
    8, 9, 11, // fifth triangle
    8, 10, 11, // sixth triangle
    //right
    12, 13, 15, // seventh triangle
    12, 14, 15, // eigth triangle
    //top
    16, 17, 18, // ninth triangle
    16, 19, 18, // ten triangle
    //bottom
    20, 21, 22, // eleven triangle
    20, 23, 22, // twelve triangle
};

void framebuffercallback(GLFWwindow *window, const int width, const int height) {
    glViewport(0, 0, width, height);
}

int main() {
    // Init GLFW
    if (!glfwInit()) return -1;
    glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR, 4);
    glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR, 6);
    glfwWindowHint(GLFW_OPENGL_PROFILE, GLFW_OPENGL_CORE_PROFILE);

    // Create window
    GLFWwindow *window = glfwCreateWindow(1920, 1080, "Triangle", glfwGetPrimaryMonitor(), NULL);
    if (!window) {
        glfwTerminate();
        return -1;
    }
    glfwMakeContextCurrent(window);
    glfwSetKeyCallback(window, key_callback);

    glfwSetFramebufferSizeCallback(window, framebuffercallback);
    // Load OpenGL functions
    if (!gladLoadGLLoader((GLADloadproc) glfwGetProcAddress)) {
        fprintf(stderr, "Failed to initialize GLAD\n");
        return -1;
    }

    unsigned int VAO;
    glGenVertexArrays(1, &VAO);
    glBindVertexArray(VAO);

    unsigned int VBO;
    glGenBuffers(1, &VBO);
    glBindBuffer(GL_ARRAY_BUFFER, VBO);
    glBufferData(GL_ARRAY_BUFFER, sizeof(vertices), vertices, GL_STATIC_DRAW);

    unsigned int EBO;
    glGenBuffers(1, &EBO);
    glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, EBO);
    glBufferData(GL_ELEMENT_ARRAY_BUFFER, sizeof(indices), indices, GL_STATIC_DRAW);

    unsigned int texture1,texture2;
    loadTextures(&texture1,"textures/ikrine_ore.png",GL_TEXTURE0);
    loadTextures(&texture2,"textures/ikrine_block.png",GL_TEXTURE1);

    SHADER shader;
    getShader(&shader, "shaders/vertex.ls", "shaders/fragment.ls");

    glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, 5 * sizeof(float), (void *) 0);
    glVertexAttribPointer(1, 2, GL_FLOAT, GL_FALSE, 5 * sizeof(float), (void *) (3 * sizeof(float)));

    // glVertexAttribPointer(2, 2, GL_FLOAT, GL_FALSE, 8 * sizeof(float), (void *) (6 * sizeof(float)));
    // glEnableVertexAttribArray(2);

    glEnableVertexAttribArray(0);
    glEnableVertexAttribArray(1);


    glEnable(GL_DEPTH_TEST);
    glPolygonMode(GL_FRONT_AND_BACK,GL_FILL);

    use(&shader);
    setInt(&shader, "texture1", 0);
    setInt(&shader, "texture2", 1);
    int n = sizeof(indices) / sizeof(unsigned int);
    while (!glfwWindowShouldClose(window)) {

        glClearColor(0.1f, 0.1f, 0.15f, 1.0f);
        glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);


        use(&shader);
        // setVec4(&shader, "color", 0.5f, 0.0f, 0.5f, 1.0f);
        // setVec2(&shader,"textCoords",1.0f,1.0f);
        glActiveTexture(GL_TEXTURE0);
        glBindTexture(GL_TEXTURE_2D, texture1);
        glActiveTexture(GL_TEXTURE1);
        glBindTexture(GL_TEXTURE_2D, texture2);
        glBindVertexArray(VAO); // Now valid VAO with vertex + index buffers + layout

        for (int i = 0; i < 10; i++) {
            mat4 transform;
            mat4 view;
            mat4 projection;

            vec3 rotateAxis = {0.3f, 0.6f, 0.9f};
            vec3 translateCoords;
            glm_vec3_make(getCoords(),translateCoords);
            translateCoords[0] += (float) i * 2.0f;

            glm_mat4_identity(transform);
            glm_mat4_identity(view);
            glm_mat4_identity(projection);
            glm_perspective(getCoords()[3], 1920.0f / 1280.0f, 0.1f, 100.0f,projection);
            glm_translate(view,translateCoords);
            glm_rotate(transform, (float) angle, rotateAxis);

            setMatrix4fv(&shader,"transform",transform[0]);
            setMatrix4fv(&shader,"view",view[0]);
            setMatrix4fv(&shader,"projection",projection[0]);
            // float mixValue = (float) sin(glfwGetTime() * 10);
            // mixValue = mixValue * 0.5f  + 0.5f;
            setFloat(&shader, "mixValue", mixValue);
            glDrawElements(GL_TRIANGLES, n, GL_UNSIGNED_INT, 0);
        }

        handleKeysPressed(window);
        glfwSwapBuffers(window);
        glfwPollEvents();
    }
    glfwTerminate();
    return 0;
}
