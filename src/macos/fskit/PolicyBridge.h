#ifndef AB_WORM_POLICY_BRIDGE_H
#define AB_WORM_POLICY_BRIDGE_H
#include <stddef.h>
#include <stdint.h>
int32_t ab_worm_check_entry_name(const uint8_t *name, size_t length);
void *ab_worm_storage_open(const char *path, char **error);
void ab_worm_storage_close(void *handle);
char *ab_worm_storage_request(void *handle, const char *request);
void ab_worm_string_free(char *value);
#endif
