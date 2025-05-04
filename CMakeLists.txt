cmake_minimum_required(VERSION 3.8)
project(OpenGL_ProjectV1)

set(CMAKE_C_STANDARD  17)
set(CMAKE_C_STANDARD_REQUIRED  ON)
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/../)
set(SOURCE_FILES main.c
        libs/shader.c
        libs/shader.h
        shaders/fragment.ls
        shaders/vertex.ls
        libs/textures.c
        libs/textures.h
        libs/special_callback.c
        libs/special_callback.h
        world/player.c
        world/player.h
        window/window.c
        window/window.h
)
set(OpenGL_GL_PREFERENCE GLVND)
set(GLFW_USE_WAYLAND 1)
set(GLFW_BUILD_TESTS Off GLFW_BUILD_EXAMPLES Off GLFW_BUILD_DOCS Off)
find_package(OpenGL REQUIRED)
find_package(glfw3 3.4 REQUIRED)
#find_package(Vulkan REQUIRED)

include_directories("/home/maxence/Documents/Dev/C.PP/Dependencies/STB_IMAGE")
include_directories("/home/maxence/Documents/Dev/C.PP/Dependencies/GLAD/include/")
include_directories("/home/maxence/Documents/Dev/C.PP/Dependencies/CGLM/include/")
aux_source_directory("/home/maxence/Documents/Dev/C.PP/Dependencies/GLAD/src/" glad_src)


add_executable(main ${SOURCE_FILES} ${glad_src})

target_link_libraries(main glfw m)
target_link_directories(main PUBLIC "/home/maxence/Documents/Dev/C.PP/Dependencies/STB_IMAGE")
target_link_directories(main PUBLIC "/home/maxence/Documents/Dev/C.PP/Dependencies/CGLM/src")