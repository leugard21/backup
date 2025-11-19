#pragma once

#include <stddef.h>

typedef struct BackupOptions {
  const char *source;
  const char *destination;
  const char *backup_name;
  int verbose;
} BackupOptions;

int backup_run(const BackupOptions *opts);