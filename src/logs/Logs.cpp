//
// Created by Sinis on 30/05/2025.
//

#include "Logs.h"

FILE** Logs::file = new FILE*{nullptr};

uint64_t get_time_ns() {
    LARGE_INTEGER frequency;
    LARGE_INTEGER counter;

    QueryPerformanceFrequency(&frequency);
    QueryPerformanceCounter(&counter);

    return (uint64_t)(counter.QuadPart * 1000000000ULL / frequency.QuadPart);
}

void Logs::init() {
    struct stat st{};
    if(stat("./logs", &st) == -1)
        mkdir("logs");
    *file = fopen("logs/logs.txt", "w");
    if (!*file) {
        printf("Failed to open log file\n");
    }
}

void Logs::log(const std::string &type, const std::string &message)  {
    if (*file) {
        fprintf(*file, "%s at %llu : %s\n",type.c_str(), get_time_ns(),message.c_str());
        fflush(*file);
    }
}

void Logs::close()  {
    if (*file) {
        fclose(*file);
        *file = nullptr;
    }
}