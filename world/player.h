//
// Created by maxence on 04/05/25.
//

#ifndef PLAYER_H
#define PLAYER_H
#include <cglm/cglm.h>

typedef struct {
    vec3 pos;
    vec3 direction;
    float yaw;
    float pitch;
    float roll;
} PLAYER;


void moveForward(float delta, PLAYER *player);
void moveRight(float delta, PLAYER *player);

void initCoords(float x, float y, float z, PLAYER *player);

void addToY(const float delta, PLAYER *player);

float* getCoords(PLAYER *player);

#endif //PLAYER_H
