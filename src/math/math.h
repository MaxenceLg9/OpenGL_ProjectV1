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

#endif //MATH_H