#ifndef SHADER_H
#define SHADER_H
#include <vec3.hpp>


class Shader{
public:
    Shader(const char *vertexPath, const char *fragmentPath);

    ~Shader();

    // use/activate the shader
    void use() const;

    void setInt(const char *name, int value) const;

    void setFloat(const char *name, float value) const;

    void setVec2(const char *name, float v1, float v2) const;

    void setVec3(const char *name, float v1, float v2,float v3) const;

    void setVec3(const char *name, const glm::vec3 &value) const;

    void setVec4(const char *name, float v1, float v2, float v3, float v4) const;

    void setMatrix4fv(const char *name, const float *matrix) const;

    static void readFile(char **buffer, const char *filename);

    static void compileShader(unsigned int *shader, const char **code, const int type, const char *typeName) ;

    unsigned int id;
};




#endif
