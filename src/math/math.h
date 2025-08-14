//
// Created by maxence on 05/05/25.
//

#ifndef MATH_H
#define MATH_H
#include "glm.hpp"
#include "ext/matrix_transform.hpp"
#include "ext/matrix_clip_space.hpp"
#include "ext/matrix_clip_space.hpp"
#include "gtc/type_ptr.hpp"
#include "gtc/noise.hpp"

struct IVec3Compare {
    bool operator()(const glm::ivec3& a, const glm::ivec3& b) const {
        if (a.x != b.x) return a.x < b.x;
        if (a.y != b.y) return a.y < b.y;
        return a.z < b.z;
    }
};

namespace Utils {

    inline float max(float a, float b) {
        if(a > b) {
            return a;
        }
        return b;
    }

    inline float min(float a, float b) {
        if(a < b) {
            return a;
        }
        return b;
    }

    inline float alpha(float t, int o) {
        if (o == 0) {
            return 1.0;
        }
        double frequency = 1.0;
        double H = 0.75;
        for (int i = 0; i < o; i++) {
            frequency *= 2.0;
        }
        return t * pow(frequency, -H);
    }

    inline float valley(float x){
        return 1.f/(0.2f * powf(4,(sinf((x) / 15.f) * 2.f + 0.1f)));
    }

    inline float mountain(float x){
        return 0.1f * powf(4,sinf(glm::perlin(glm::vec3(x * 0.01f,x * 0.01f,0.0f))) * 2.f + 5);
    }

    inline float terrain2(float x, float y){
        return Utils::valley(x) * glm::perlin(glm::vec2(x,y) * 0.01f);
    }

    inline double noised_terrain_default(float x, float y) {
        float ret = 0.0;
        float frequency = 0.005f;
        for (int i = 0; i < 4; i++) {
            ret += (float) Utils::alpha(ret, i) * glm::perlin(glm::vec3((float)x * frequency, (float)y * frequency, 0.0));
            frequency *= 2.0;
        }
        return ret;
    }

    inline double terrain(float x, float y) {
//        return noised_terrain_default(x,y) * Utils::mountain(x + y) + 200;
        return powf(5,noised_terrain_default(x,y) * 5.f + 1);
    }

}

#endif //MATH_H