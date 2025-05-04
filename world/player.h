//
// Created by maxence on 04/05/25.
//

#ifndef PLAYER_H
#define PLAYER_H

void addToZ(float delta);
void addToX(float delta);
void addToY(float delta);
void addToMouse(float deltaX,float deltaY);

void initCoords(const float x, const float y, const float z, const float mX, const float mY);

float* getCoords();

#endif //PLAYER_H
