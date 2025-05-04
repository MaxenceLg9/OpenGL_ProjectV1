//
// Created by maxence on 04/05/25.
//

#ifndef PLAYER_H
#define PLAYER_H

void addToZ(float delta);
void addToX(float delta);
void addToY(float delta);
void addToW(float delta);

void initCoords(float x, float y, float z, float w);

float* getCoords();

#endif //PLAYER_H
