#include "backup.h"
#include "fs.h"

#include <errno.h>
#include <limits.h>
#include <linux/limits.h>
#include <stdio.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/wait.h>
#include <unistd.h>

static int run_directory_backup(const BackupOptions *opts) {
  char backup_path[PATH_MAX];
  int n = snprintf(backup_path, sizeof(backup_path), "%s/%s", opts->destination,
                   opts->backup_name);
  if (n < 0 || n >= (int)sizeof(backup_path)) {
    fprintf(stderr, "backup: backup path too long\n");
    return -1;
  }

  if (opts->verbose) {
    fprintf(stderr, "backup: from '%s'\n", opts->source);
    fprintf(stderr, "backup: to   '%s'\n", backup_path);
  }

  if (fs_ensure_directory(backup_path, opts->verbose) != 0)
    return -1;

  if (fs_copy_tree(opts->source, backup_path, opts->verbose) != 0) {
    fprintf(stderr, "backup: copy failed\n");
    return -1;
  }

  return 0;
}

static int run_compressed_backup(const BackupOptions *opts) {
  char archive_path[PATH_MAX];
  int n = snprintf(archive_path, sizeof(archive_path), "%s/%s.tar.gz",
                   opts->destination, opts->backup_name);
  if (n < 0 || n >= (int)sizeof(archive_path)) {
    fprintf(stderr, "backup: archive path too long\n");
    return -1;
  }

  if (opts->verbose) {
    fprintf(stderr, "backup: creating archive '%s'\n", archive_path);
    fprintf(stderr, "backup: source directory '%s'\n", opts->source);
  }

  pid_t pid = fork();
  if (pid < 0) {
    fprintf(stderr, "backup: fork failed: %s\n", strerror(errno));
    return -1;
  }

  if (pid == 0) {
    execlp("tar", "tar", "-czf", archive_path, "-C", opts->source, ".",
           (char *)NULL);
    fprintf(stderr, "backup: failed to exec 'tar': %s\n", strerror(errno));
    _exit(127);
  }

  int status = 0;
  if (waitpid(pid, &status, 0) < 0) {
    fprintf(stderr, "backup: waitpid failed: %s\n", strerror(errno));
    return -1;
  }

  if (!WIFEXITED(status) || WEXITSTATUS(status) != 0) {
    fprintf(stderr, "backup: tar failed (status=%d)\n", status);
    return -1;
  }

  if (opts->verbose) {
    fprintf(stderr, "backup: archive created successfully\n");
  }

  return 0;
}

int backup_run(const BackupOptions *opts) {
  if (!opts || !opts->source || !opts->destination || !opts->backup_name) {
    fprintf(stderr, "backup: invalid options\n");
    return -1;
  }

  struct stat st_src;
  struct stat st_dst;

  if (stat(opts->source, &st_src) != 0) {
    fprintf(stderr, "backup: source '%s' does not exist: %s\n", opts->source,
            strerror(errno));
    return -1;
  }
  if (!S_ISDIR(st_src.st_mode)) {
    fprintf(stderr, "backup: source is not a directory: %s\n", opts->source);
    return -1;
  }

  if (stat(opts->destination, &st_dst) != 0) {
    fprintf(stderr, "backup: destination '%s' does not exist: %s\n",
            opts->destination, strerror(errno));
    return -1;
  }
  if (!S_ISDIR(st_dst.st_mode)) {
    fprintf(stderr, "backup: destination is not a directory: %s\n",
            opts->destination);
    return -1;
  }

  if (opts->compress) {
    return run_compressed_backup(opts);
  }

  return run_directory_backup(opts);
}