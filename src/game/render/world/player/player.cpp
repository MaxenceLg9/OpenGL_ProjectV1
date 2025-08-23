//
// Created by maxence on 04/05/25.
//

#include "player.h"
#include <cstdio>

#include "../../../../math/math.h"
#include "../../../../utils/logs/Logs.h"

Player::~Player() {
    Logs::debug("Destroying player");
}


void Player::moveForward(const float delta) {
    //add the offset to the player position : move
    // printf("Moving forward: %f\n", glm::length(this->direction));
    this->pos += this->direction * delta * 3.5f * getSpeed();
    printf("Pos : %f,%f,%f\n", this->pos.x, this->pos.y, this->pos.z);
    printf("Direction : %f,%f,%f\n", this->direction.x, this->direction.y, this->direction.z);
}

void Player::moveRight(const float delta) {
    glm::vec3 right = glm::normalize(glm::cross(this->direction, this->up));
    // printf("Moving right: %f\n", glm::length(right));
    this->pos = this->pos + right * delta * 3.5f * getSpeed();
    printf("Pos : %f,%f,%f\n", this->pos.x, this->pos.y, this->pos.z);
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

void Player::addFov(float fov) {
    this->fov -=fov;
    if (this->fov < 30.0f) {
        this->fov = 30.0f;
    } else if (this->fov > 140.0f) {
        this->fov = 140.0f;
    }
}

float Player::getFov() const {
    return this->fov;
}


Player::Player(const float x, const float y, const float z): deltaTime(0), fov(140.0f), pos(x,y,z), roll(0.0f), direction(glm::vec3(0.0f, 0.0f, -1.0f)), up(0.0f,1.0f,0.0f) {
    printf("Creating player at %f,%f,%f\n", x, y, z);
}

void Player::setDeltaTime(double delta) {
    this->deltaTime = delta;
}


glm::vec3 Player::getCoords() const {
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
