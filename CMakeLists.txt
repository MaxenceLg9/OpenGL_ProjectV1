cmake_minimum_required(VERSION 3.13)
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
        src/game/world/player/player.cpp
        src/game/world/player/player.h
        src/display/window/window.cpp
        src/display/window/window.h
        src/math/math.cpp
        src/math/math.h
        src/game/world/chunk/chunk.cpp
        src/game/world/chunk/chunk.h
        src/game/world/block/block.cpp
        src/game/world/block/block.h
        src/display/model/model.h
        src/display/model/mesh/mesh.h
        src/display/model/model.cpp
        src/display/model/mesh/mesh.cpp
        src/display/window/cursor/cursor.cpp
        src/display/window/cursor/cursor.h
        src/game/world/light/light.cpp
        src/game/world/light/light.h
        src/game/world/World.cpp
        src/game/world/World.h
        src/logs/Logs.cpp
        src/logs/Logs.h
        src/display/model/mesh/vertex/Vertex.h
        src/display/model/mesh/vertex/Vertex.cpp
        src/display/model/mesh/ChunkMesh.cpp
        src/display/model/mesh/ChunkMesh.h
        src/configs/Configs.cpp
        src/configs/Configs.h
        src/display/model/mesh/texture/TextureArray.cpp
        src/display/model/mesh/texture/TextureArray.h
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

target_link_libraries(main m)
if(WIN32)
    target_link_libraries(main glfw3)
else()
    target_link_libraries(main glfw)
endif()
target_link_directories(main PUBLIC "../../Dependencies/STB_IMAGE")
target_link_directories(main PUBLIC "../../Dependencies/GLFW/lib/")
target_link_directories(main PUBLIC "../../Dependencies/GLM/glm/")
#target_link_directories(main PUBLIC "../../Dependencies/CGLM/src")add_executable(terrain_test src/test_terraingen.cpp)


add_executable(test_terraingen src/test_terraingen.cpp src/math/math.cpp src/math/math.h)