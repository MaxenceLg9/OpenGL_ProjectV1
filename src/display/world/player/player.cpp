//
// Created by maxence on 04/05/25.
//

#include "player.h"
#include <cstdio>

#include "../../../math/math.h"

void Player::moveForward(const float delta) {
    //add the offset to the player position : move
    printf("Moving forward: %f\n", glm::length(this->direction));
    this->pos += this->direction * delta * 3.5f * getSpeed();
//    printf("pos: %f %f %f\n", this->pos.x, this->pos.y, this->pos.z);
}

void Player::moveRight(const float delta) {
    glm::vec3 right = glm::cross(this->direction, this->up);
    printf("Moving right: %f\n", glm::length(right));
    this->pos = this->pos + right * delta * 3.5f * getSpeed();

//    printf("pos: %f %f %f\n", this->pos.x, this->pos.y, this->pos.z);
}

void Player::moveUp(const float delta) {

    // Rotate baseUp around front vector (XZ plane) by roll angle to get the rolled-up vector
    printf("Length rolledUp: %f\n", glm::length(this->up));
    // Move along local-up
    this->pos += this->up * delta * 2.5f * getSpeed();
}

float Player::getSpeed() const{
    double speed = deltaTime;
    for(const auto& [key, i] : speedMultiplier) {
        speed *= i;
    }
    return (float) speed;
}


Player::Player(const float x, const float y, const float z) {
    this->direction = glm::normalize(glm::vec3(0.0f, 0.0f, -1.0f));
    this->pos = glm::vec3(x, y, z);
    this->up = glm::vec3(0.0f, 1.0f, 0.0f);
    this->roll = 0.0f;
}

void Player::setDeltaTime(double delta) {
    this->deltaTime = delta;
}


glm::vec3 Player::getCoords() {
    return this->pos;
}

glm::vec3 Player::getDirection() const {
    return this->direction;
}

void Player::makeRoll(float angle) {
    roll += angle;
    this->computeUp(angle);
}

glm::vec3 Player::getUp() const {
    return this->up;
}

void Player::moveCamera(float xoffset, float yoffset) {
    // Create rotation matrices
    glm::vec3 front(this->direction.x, 0.0f, this->direction.z);
    glm::vec3 baseUp(0.0f, 1.0f, 0.0f);

    // Rotate baseUp around front vector (XZ plane) by roll angle to get the rolled-up vector
    this->direction = glm::rotate(glm::mat4(1.0f), glm::radians(xoffset), this->up) * glm::vec4(this->direction, 0.0f);
    // Compute the right vector
    this->direction = glm::rotate(glm::mat4(1.0f), glm::radians(-yoffset), glm::cross(this->direction,this->up)) * glm::vec4(this->direction, 0.0f);
    // Re compute the up vector
    this->computeUp();
}

void Player::computeUp(){
    this->up = glm::normalize(glm::rotate(glm::mat4(1.0f), glm::radians(roll), this->direction) * glm::vec4(glm::vec3(0.0f,1.0f,0.0f) ,0.0f));
}
void Player::computeUp(float angle){
    this->up = glm::rotate(glm::mat4(1.0f), glm::radians(angle), this->direction) * glm::vec4(this->up,1.f);
    this->up = glm::normalize(this->up);
//    printf("up: %f %f %f\n", this->up.x, this->up.y, this->up.z);
}



void Player::addSpeedMultiplier(int key,double multi) {
    this->speedMultiplier.try_emplace(key,multi);
}


void Player::removeSpeedMultiplier(int key) {
    this->speedMultiplier.erase(key);
}
