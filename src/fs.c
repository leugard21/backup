#include "fs.h"

#include <dirent.h>
#include <errno.h>
#include <fcntl.h>
#include <limits.h>
#include <linux/limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <unistd.h>

static int copy_file(const char *src, const char *dst, int verbose) {
  int src_fd = -1;
  int dst_fd = -1;
  int rc = -1;

  src_fd = open(src, O_RDONLY);
  if (src_fd < 0) {
    fprintf(stderr, "backup: failed to open '%s' for reading: %s\n", src,
            strerror(errno));
    goto out;
  }

  struct stat st;
  if (fstat(src_fd, &st) != 0) {
    fprintf(stderr, "backup: fstat failed on '%s': %s\n", src, strerror(errno));
    goto out;
  }

  mode_t mode = st.st_mode & 0777;

  dst_fd = open(dst, O_WRONLY | O_CREAT | O_TRUNC, mode);
  if (dst_fd < 0) {
    fprintf(stderr, "backup: failed to open '%s' for writing: %s\n", dst,
            strerror(errno));
    goto out;
  }

  if (verbose)
    fprintf(stderr, "copy: %s -> %s\n", src, dst);

  char buf[64 * 1024];
  for (;;) {
    ssize_t n = read(src_fd, buf, sizeof(buf));
    if (n < 0) {
      fprintf(stderr, "backup: read error on '%s': %s\n", src, strerror(errno));
      goto out;
    }
    if (n == 0)
      break;

    char *p = buf;
    ssize_t to_write = n;
    while (to_write > 0) {
      ssize_t w = write(dst_fd, p, (size_t)to_write);
      if (w < 0) {
        fprintf(stderr, "backup: write error on '%s': %s\n", dst,
                strerror(errno));
        goto out;
      }
      to_write -= w;
      p += w;
    }
  }

  rc = 0;

out:
  if (src_fd >= 0) {
    close(src_fd);
  }
  if (dst_fd >= 0) {
    close(dst_fd);
  }
  return rc;
}

int fs_ensure_directory(const char *path, int verbose) {
  struct stat st;
  if (stat(path, &st) == 0) {
    if (S_ISDIR(st.st_mode)) {
      if (verbose) {
        fprintf(stderr, "dir exists: %s\n", path);
      }
      return 0;
    }
    fprintf(stderr, "backup: path exists and is not a directory: %s\n", path);
    return -1;
  }

  if (mkdir(path, 0755) != 0) {
    fprintf(stderr, "backup: failed to create directory '%s': %s\n", path,
            strerror(errno));
    return -1;
  }

  if (verbose)
    fprintf(stderr, "mkdir: %s\n", path);

  return 0;
}

static int copy_tree_recursive(const char *src, const char *dst, int verbose) {
  DIR *dir = opendir(src);
  if (!dir) {
    fprintf(stderr, "backup: failed to open directory '%s': %s\n", src,
            strerror(errno));
    return -1;
  }

  struct dirent *entry;
  int rc = 0;

  while ((entry = readdir(dir)) != NULL) {
    const char *name = entry->d_name;

    if (strcmp(name, ".") == 0 || strcmp(name, "..") == 0)
      continue;

    char src_path[PATH_MAX];
    char dst_path[PATH_MAX];

    if (snprintf(src_path, sizeof(src_path), "%s/%s", src, name) >=
        (int)sizeof(src_path)) {
      fprintf(stderr, "backup: source path too long: %s/%s\n", src, name);
      rc = -1;
      break;
    }

    if (snprintf(dst_path, sizeof(dst_path), "%s/%s", dst, name) >=
        (int)sizeof(dst_path)) {
      fprintf(stderr, "backup: destination path too long: %s/%s\n", dst, name);
      rc = -1;
      break;
    }

    struct stat st;
    if (stat(src_path, &st) != 0) {
      fprintf(stderr, "backup: stat failed for '%s': %s\n", src_path,
              strerror(errno));
      rc = -1;
      break;
    }

    if (S_ISDIR(st.st_mode)) {
      if (fs_ensure_directory(dst_path, verbose) != 0) {
        rc = -1;
        break;
      }
      if (copy_tree_recursive(src_path, dst_path, verbose) != 0) {
        rc = -1;
        break;
      }
    } else if (S_ISREG(st.st_mode)) {
      if (copy_file(src_path, dst_path, verbose) != 0) {
        rc = -1;
        break;
      }
    } else if (S_ISLNK(st.st_mode)) {
      fprintf(stderr, "backup: skipping symlink '%s'\n", src_path);
    } else {
      fprintf(stderr, "backup: skipping unsupported file type '%s'\n",
              src_path);
    }
  }

  closedir(dir);
  return rc;
}

int fs_copy_tree(const char *src_root, const char *dst_root, int verbose) {
  return copy_tree_recursive(src_root, dst_root, verbose);
}
