#include <stdlib.h>

/* Touch 32 MiB to independently check the RSS observer's units and admission. */
int main(void) {
    const size_t bytes = 32 * 1024 * 1024;
    volatile unsigned char *memory = malloc(bytes);
    if (!memory) return 2;
    for (size_t offset = 0; offset < bytes; offset += 4096) memory[offset] = 1;
    int result = memory[bytes - 4096] == 1 ? 0 : 2;
    free((void *)memory);
    return result;
}
