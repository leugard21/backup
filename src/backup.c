#include "backup.h"
#include <stdio.h>

int backup_run(const BackupOptions *opts) {
  if (!opts || !opts->source || !opts->destination || !opts->backup_name) {
    fprintf(stderr, "backup: invalid options\n");
    return -1;
  }

  if (opts->verbose) {
    fprintf(stderr, "backup: from '%s' to '%s' as '%s'\n", opts->source,
            opts->destination, opts->backup_name);
    fprintf(stderr, "backup: NOTE: core copy logic not implemented yet.\n");
  }

  return 0;
}