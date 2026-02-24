//
// Created by maxence on 07/05/25.
//

#include "block.h"

Block::Block(float ambient, float diffuse, float specular) : ambient(ambient), diffuse(diffuse), specular(specular) {
}

float Block::getAmbient() const { return ambient; }
float Block::getDiffuse() const { return diffuse; }
float Block::getSpecular() const { return specular; }