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

struct IVec3Compare {
    bool operator()(const glm::ivec3& a, const glm::ivec3& b) const {
        if (a.x != b.x) return a.x < b.x;
        if (a.y != b.y) return a.y < b.y;
        return a.z < b.z;
    }
};

namespace Utils {

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

}

#endif //MATH_H