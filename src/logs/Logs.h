//
// Created by Sinis on 30/05/2025.
//

#ifndef LOGS_H
#define LOGS_H

#define WORLD_SIZE 1

#include <cstdio>
#include <cstdint>
#include <unistd.h>
#include <string>
#include <ctime>
#include <sys/stat.h>

uint64_t get_time_ns();

#if defined(_WIN32) || defined(_WIN64)
#include <windows.h>
#endif


class Logs {
public:
    static void init();

    static void log(const std::string &type, const std::string &message);

    static void close();
private:
    static FILE** file;
};


#endif //LOGS_H
