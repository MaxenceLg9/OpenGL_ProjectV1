//
// Created by maxence on 04/05/25.
//

#ifndef PLAYER_H
#define PLAYER_H

#include <vector>
#include <map>
#include "glm.hpp"
#include "vec3.hpp"

class Player {
public:
    void moveForward(float delta);

    void moveRight(float delta);

    Player(float x, float y, float z);
    ~Player();

    void moveUp(float delta);

    glm::vec3 getCoords() const;

    glm::vec3 getDirection() const;

    glm::vec3 getUp() const;

    void makeRoll(float angle);

    void moveCamera(float xoffset, float yoffset);

    void setDeltaTime(double delta);

    void addSpeedMultiplier(int key,double multi);

    void removeSpeedMultiplier(int key);

    void setFov(float fov);

    float getFov() const;

private:
    glm::vec3 pos, direction,up;
    double deltaTime;
    float roll;
    std::map<int,double> speedMultiplier;
    float fov;

    float getSpeed() const;

    void computeUp();

    void computeUp(float angle);
};



#endif //PLAYER_H
