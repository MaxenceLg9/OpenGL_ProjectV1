//
// Created by maxence on 04/05/25.
//

#include "player.h"
#include "cglm/cglm.h"

#include "../../../math/math.h"

void Player::moveForward(const float delta) {
    //add the offset to the player position : move
    this->pos += this->direction * delta;
    printf("pos: %f %f %f\n", this->pos.x, this->pos.y, this->pos.z);
}

void Player::moveRight(const float delta) {
    glm::vec3 right, up(0.0f,1.0f,0.0f), front(direction.x,0.0f,direction.z);
    up = glm::rotate(glm::mat4(1.0f), glm::radians(this->getRoll()), front) * glm::vec4(up,0.0f);
    right = glm::cross(this->direction, up);
    right = right * delta;
    printf("Moving right: %f\n", glm::length(right));
    this->pos = this->pos + right;
    printf("pos: %f %f %f\n", this->pos.x, this->pos.y, this->pos.z);
}

void Player::moveUp(const float delta) {
    glm::vec3 front(this->direction.x, 0.0f, this->direction.z);
    glm::vec3 baseUp(0.0f, 1.0f, 0.0f);

    // Rotate baseUp around front vector (XZ plane) by roll angle to get the rolled-up vector
    glm::vec3 rolledUp = glm::rotate(glm::mat4(1.0f), glm::radians(this->getRoll()), front) * glm::vec4(baseUp, 0.0f);

    // Compute the right vector
    glm::vec3 right = glm::normalize(glm::cross(this->direction, rolledUp));

    // Compute final local-up vector (cross of right and direction)
    glm::vec3 localUp = glm::normalize(glm::cross(right, this->direction));

    // Move along local-up
    this->pos += localUp * delta;
}


Player::Player(const float x, const float y, const float z) {
    this->direction = glm::normalize(glm::vec3(0.0f, 0.0f, -1.0f));
    this->pos = glm::vec3(x, y, z);
    this->roll = 90.0f;
}


glm::vec3 Player::getCoords() {
    return this->pos;
}

glm::vec3 Player::getDirection() {
    return this->direction;
}

void Player::makeRoll(float angle) {
    this->roll = this->roll + angle;
    printf("roll: %f\n", getRoll());

}

float Player::getRoll() const {
    return roll - 90.0f;
}

void Player::moveCamera(float xoffset, float yoffset) {
    // Create rotation matrices
    glm::mat4 yawMat(1.0f), pitchMat(1.0f);

    // Rotate around local up (roll-aware) for yaw
    glm::vec3 up;
    glm::vec3 baseUp = {0.0f, 1.0f, 0.0f};
    glm::mat4 rollMat(1.0f);
    rollMat = glm::rotate(rollMat, glm::radians(this->getRoll()),this->getDirection());
    up = rollMat * glm::vec4(baseUp,0.0f);
    yawMat = glm::rotate(yawMat, xoffset, up);

    // Rotate around local right vector for pitch
    glm::vec3 right;
    right = glm::normalize(glm::cross(up, this->getDirection()));
    pitchMat = glm::rotate(pitchMat, yoffset, glm::normalize(glm::cross(up, this->getDirection())));

    // Apply both rotations to direction
    this->direction = glm::normalize(yawMat * pitchMat * glm::vec4(this->getDirection(),0.0f));
}
