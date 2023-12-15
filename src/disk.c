#include <stdint.h>
#include <stdbool.h>
#include <sys/statvfs.h>

#include <stdio.h>

bool disk_stat(const char* path, uint64_t* total, uint64_t* free)
{
    struct statvfs stat;
    if (statvfs(path, &stat) < 0)
    {
        return false;
    }

    if (total != NULL) *total = stat.f_frsize * stat.f_blocks;
    if (free != NULL) *free = stat.f_frsize * stat.f_bfree;
    return true;
}