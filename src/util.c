#include "util.h"

#include <stdio.h>
#include <time.h>

int util_make_default_backup_name(char *buf, size_t buf_size) {
  if (!buf || buf_size == 0)
    return -1;

  time_t now = time(NULL);
  if (now == (time_t)-1)
    return -1;

  struct tm *tm_ptr = localtime(&now);
  if (!tm_ptr) {
    return -1;
  }

  size_t written = strftime(buf, buf_size, "backup-%Y%m%d-%H%M%S", tm_ptr);
  if (written == 0)
    return -1;

  return 0;
}