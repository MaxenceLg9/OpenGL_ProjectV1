cmake_minimum_required(VERSION 3.8)
project(OpenGL_ProjectV1)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED  ON)
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/../src/)
set(SOURCE_FILES
        src/main.cpp
        src/display/model/mesh/shader/shader.cpp
        src/display/model/mesh/shader/shader.h
        src/display/model/mesh/texture/textures.cpp
        src/display/model/mesh/texture/textures.h
        src/display/callback/callback.cpp
        src/display/callback/callback.h
        src/display/world/player/player.cpp
        src/display/world/player/player.h
        src/display/window/window.cpp
        src/display/window/window.h
        src/math/math.cpp
        src/math/math.h
        src/display/world/chunk/chunk.cpp
        src/display/world/chunk/chunk.h
        src/display/world/block/block.cpp
        src/display/world/block/block.h
        src/display/model/model.h
        src/display/model/mesh/mesh.h
        src/display/model/model.cpp
        src/display/model/mesh/mesh.cpp
        src/display/window/cursor/cursor.cpp
        src/display/window/cursor/cursor.h
)
set(OpenGL_GL_PREFERENCE GLVND)
set(GLFW_USE_WAYLAND 1)
set(GLFW_BUILD_TESTS Off GLFW_BUILD_EXAMPLES Off GLFW_BUILD_DOCS Off)
find_package(OpenGL REQUIRED)
#find_package(glfw3 3.4 REQUIRED)
#find_package(Vulkan REQUIRED)

include_directories("../../Dependencies/STB_IMAGE")
include_directories("../../Dependencies/GLFW/include/")
include_directories("../../Dependencies/GLAD/include/")
include_directories("../../Dependencies/CGLM/include/")
include_directories("../../Dependencies/GLM/glm/")
aux_source_directory("../../Dependencies/GLAD/src/" glad_src)


add_executable(main ${SOURCE_FILES} ${glad_src})

target_link_libraries(main glfw3 m)
target_link_directories(main PUBLIC "../../Dependencies/STB_IMAGE")
target_link_directories(main PUBLIC "../../Dependencies/GLFW/lib/")
target_link_directories(main PUBLIC "../../Dependencies/GLM/glm/")
#target_link_directories(main PUBLIC "../../Dependencies/CGLM/src")