//
// Created by maxence on 04/05/25.
//

#include "player.h"

#include <stdio.h>
#include <stdlib.h>
#include <cglm/vec3.h>

void moveForward(const float delta, PLAYER *player) {
    vec3 offset;
    // multiply the direction by the delta to have the amplitude of the movement
    glm_vec3_scale(player->direction, delta, offset);
    //add the offset to the player position : move
    glm_vec3_add(player->pos, offset, player->pos);
}

void moveRight(const float delta, PLAYER *player) {
    vec3 right, up = {0.0f, 1.0f, 0.0f};
    glm_vec3_cross(player->direction, up, right); // front x up = right
    glm_normalize(right);
    glm_vec3_scale(right, delta, right);
    glm_vec3_add(player->pos, right, player->pos);
}

void addToY(const float delta, PLAYER *player) {
    player->pos[1] += delta;
}

void initCoords(const float x, const float y, const float z, PLAYER *player) {
    glm_vec3_copy((vec3){x, y, z}, player->pos);
    glm_vec3_copy((vec3){0.0f, 0.0f, -1.0f}, player->direction);
}


float* getCoords(PLAYER *player) {
    return player->pos;
}