//
// Created by Sinis on 14/08/2025.
//
#include <cstdio>
#include "math/math.h"

int main(int argc, char **argv) {
    // Initialize the world

    for (int i = 0; i < 200; i++) {
        printf("i : %d, Default: %f, Terrain : %f, Mountain : %f\n", i, Utils::noised_terrain_default(i,i), Utils::terrain(i,i), Utils::mountain(i + i));
    }

    return 0;
}