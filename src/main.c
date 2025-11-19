#include <getopt.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include "backup.h"
#include "util.h"

static void print_usage(const char *prog) {
  fprintf(
      stderr,
      "Usage: %s [-n name] [-c] [-v] [-h] <source> <destination>\n"
      "\n"
      "Options:\n"
      "  -n name   Custom backup name (base name for dir or archive)\n"
      "  -c        Create compressed tar.gz archive instead of directory copy\n"
      "  -v        Verbose output\n"
      "  -h        Show this help message\n",
      prog);
}

int main(int argc, char *argv[]) {
  int opt;
  const char *custom_name = NULL;
  int verbose = 0;
  int compress = 0;

  while ((opt = getopt(argc, argv, "n:cvh")) != -1) {
    switch (opt) {
    case 'n':
      custom_name = optarg;
      break;
    case 'c':
      compress = 1;
      break;
    case 'v':
      verbose = 1;
      break;
    case 'h':
      print_usage(argv[0]);
      return 0;
    default:
      print_usage(argv[0]);
      return 1;
    }
  }

  if (argc - optind != 2) {
    fprintf(stderr, "backup: expected <source> and <destination>\n");
    print_usage(argv[0]);
    return 1;
  }

  const char *source = argv[optind];
  const char *destination = argv[optind + 1];

  char name_buf[BACKUP_NAME_MAX];

  const char *backup_name = custom_name;
  if (!backup_name) {
    if (util_make_default_backup_name(name_buf, sizeof(name_buf)) != 0) {
      fprintf(
          stderr,
          "backup: failed to generate default backup name, using 'backup'\n");
      strncpy(name_buf, "backup", sizeof(name_buf));
      name_buf[sizeof(name_buf) - 1] = '\0';
    }
    backup_name = name_buf;
  }

  BackupOptions opts = {
      .source = source,
      .destination = destination,
      .backup_name = backup_name,
      .verbose = verbose,
      .compress = compress,
  };

  int rc = backup_run(&opts);
  if (rc != 0) {
    if (!verbose) {
      fprintf(stderr, "backup: backup failed (code %d)\n", rc);
    }
    return rc;
  }

  if (verbose) {
    fprintf(stderr, "backup: completed successfully\n");
  }

  return 0;
}
