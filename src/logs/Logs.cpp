//
// Created by Sinis on 30/05/2025.
//

#include "Logs.h"

#if defined(_WIN32) || defined(_WIN64)
#include <windows.h>
uint64_t get_time_ns() {
    LARGE_INTEGER frequency;
    LARGE_INTEGER counter;

    QueryPerformanceFrequency(&frequency);
    QueryPerformanceCounter(&counter);

    return (uint64_t)(counter.QuadPart * 1000000000ULL / frequency.QuadPart);
}
#else
uint64_t get_time_ns() {
    timespec ts = {};
    clock_gettime(CLOCK_REALTIME, &ts);

    return (uint64_t) ts.tv_nsec;
}
#endif


FILE** Logs::file = new FILE*{nullptr};

void Logs::init() {
    struct stat st{};
    if(stat("./logs", &st) == -1)
#if defined(_WIN32) || defined(_WIN64)
        mkdir("logs");
#else
            mkdir("logs", 755);
#endif
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