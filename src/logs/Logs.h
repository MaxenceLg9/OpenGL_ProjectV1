//
// Created by Sinis on 30/05/2025.
//

#ifndef LOGS_H
#define LOGS_H

#define WORLD_SIZE 1


#include <cstdio>
#include <ctime>
#include <string>
#include <sys/types.h>
#include <sys/stat.h>
#include <unistd.h>
#include <windows.h>
#include <cstdint>
#include <cstdio>



class Logs {
public:
    static void init();

    static void log(const std::string &type, const std::string &message);

    static void close();
private:
    static FILE** file;
};


#endif //LOGS_H
