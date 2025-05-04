//
// Created by maxence on 04/05/25.
//

#include "player.h"

#include <stdlib.h>

float coords[5] = {0.0f, 0.0f, 0.0f, 1.0f,1.0f};

void addToZ(const float delta) {
    coords[2] += delta;
}

void addToX(const float delta) {
    coords[0] += delta;
}

void addToY(const float delta) {
    coords[1] += delta;
}

void addToMouse(const float deltaX, const float deltaY) {
    coords[3] += deltaX;
    coords[4] += deltaY;
}

float* getCoords() {
    return coords;
}

void initCoords(const float x, const float y, const float z, const float mX, const float mY) {
    coords[0] = x;
    coords[1] = y;
    coords[2] = z;
    coords[3] = mX;
    coords[4] = mY;
}