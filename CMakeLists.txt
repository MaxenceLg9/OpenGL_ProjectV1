cmake_minimum_required(VERSION 3.13)
project(OpenGL_ProjectV1)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED  ON)
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/../src/)
set(SOURCE_FILES
        src/main.cpp
        src/game/render/mesh/shader/shader.cpp
        src/game/render/mesh/shader/shader.h
        src/game/render/mesh/texture/textures.cpp
        src/game/render/mesh/texture/textures.h
        src/game/render/callback/callback.cpp
        src/game/render/callback/callback.h
        src/game/render/world/player/player.cpp
        src/game/render/world/player/player.h
        src/game/render/window.cpp
        src/game/render/window.h
        src/math/math.cpp
        src/math/math.h
        src/game/render/world/chunk/chunk.cpp
        src/game/render/world/chunk/chunk.h
        src/game/render/world/block/block.cpp
        src/game/render/world/block/block.h
        src/game/render/mesh/mesh.h
        src/game/render/mesh/mesh.cpp
        src/game/render/gui/cursor/cursor.cpp
        src/game/render/gui/cursor/cursor.h
        src/game/render/world/light/light.cpp
        src/game/render/world/light/light.h
        src/game/render/world/World.cpp
        src/game/render/world/World.h
        src/utils/logs/Logs.cpp
        src/utils/logs/Logs.h
        src/game/render/mesh/vertex/Vertex.h
        src/game/render/mesh/vertex/Vertex.cpp
        src/game/render/mesh/ChunkMesh.cpp
        src/game/render/mesh/ChunkMesh.h
        src/configs/Configs.cpp
        src/configs/Configs.h
        src/game/render/mesh/texture/TextureArray.cpp
        src/game/render/mesh/texture/TextureArray.h
        src/utils/OpenGL/OpenGL.cpp
        src/utils/OpenGL/OpenGL.h
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
    target_link_directories(main PUBLIC "../../Dependencies/GLFW/lib")
else()
    target_link_libraries(main glfw)
    target_link_directories(main PUBLIC "../../Dependencies/GLFW/glfw-3.4/lib")
endif()
target_link_directories(main PUBLIC "../../Dependencies/STB_IMAGE")
target_link_directories(main PUBLIC "../../Dependencies/GLM/glm/")
#target_link_directories(main PUBLIC "../../Dependencies/CGLM/src")add_executable(terrain_test src/test_terraingen.cpp)


add_executable(test_terraingen src/test_terraingen.cpp src/math/math.cpp src/math/math.h)