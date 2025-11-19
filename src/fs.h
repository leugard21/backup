#pragma once

int fs_ensure_directory(const char *path, int verbose);
int fs_copy_tree(const char *src_root, const char *dst_root, int verbose);
