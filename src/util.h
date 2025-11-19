#pragma once

#include <stddef.h>

#define BACKUP_NAME_MAX 64

int util_make_default_backup_name(char *buf, size_t buf_size);