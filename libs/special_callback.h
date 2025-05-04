//
// Created by maxence on 03/05/25.
//

#ifndef SPECIAL_CALLBACK_H
#define SPECIAL_CALLBACK_H

extern float mixValue;
extern double angle;

enum KEY_STATUS{
    PRESSED = 1,
    RELEASED = 0
};

typedef struct  {
    enum KEY_STATUS status;
    void (* function)();
    int fnNumber;
} KEYS;



void key_callback(GLFWwindow* window, int key, int scancode, int action, int mods);

void handleKeysPressed(GLFWwindow *w);

#endif //SPECIAL_CALLBACK_H
