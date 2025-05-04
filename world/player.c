//
// Created by maxence on 04/05/25.
//

#include "player.h"

#include <stdlib.h>

float coords[4];

void addToZ(const float delta) {
    coords[2] += delta;
}

void addToX(const float delta) {
    coords[0] += delta;
}

void addToY(const float delta) {
    coords[1] += delta;
}

void addToW(const float delta) {
    coords[3] += delta;
}

float* getCoords() {
    return coords;
}

void initCoords(const float x, const float y, const float z, const float w) {
    coords[0] = x;
    coords[1] = y;
    coords[2] = z;
    coords[3] = w;
}