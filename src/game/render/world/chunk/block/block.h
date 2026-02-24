//
// Created by maxence on 07/05/25.
//

#ifndef CUBE_H
#define CUBE_H

enum BlockType {
    AIR = 0,
    DIRT = 1,
    STONE = 2,
    DEEPSLATE = 3,
    PLATINUM_ORE = 4,
    IKRINE_BLOCK = 5,
    IKRINE_ORE = 6,
    STONE_BRICKS = 7
};

class Block{
public:
    Block(float ambient, float diffuse, float specular);
    float getAmbient() const;
    float getDiffuse() const;
    float getSpecular() const;
private:

    float ambient,diffuse,specular;

};

#endif //CUBE_H
