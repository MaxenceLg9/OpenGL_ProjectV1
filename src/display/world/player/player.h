//
// Created by maxence on 04/05/25.
//

#ifndef PLAYER_H
#define PLAYER_H
#include "glm.hpp"
#include "vec3.hpp"

class Player {
public:
    void moveForward(float delta);

    void moveRight(float delta);

    Player(float x, float y, float z);

    void moveUp(float delta);

    glm::vec3 getCoords();

    glm::vec3 getDirection();

    float getRoll() const;

    void makeRoll(float angle);

    void moveCamera(float xoffset, float yoffset);

private:
    glm::vec3 pos, direction;
    float roll;

};



#endif //PLAYER_H
