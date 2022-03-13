#include <stdio.h>

void* calloc(size_t, size_t);
void free(void *);
int posix_memalign(void**, size_t, size_t);

int main() {
    void* ptr = NULL;
    size_t alignment = 512;
    printf("status: %d, alignment: %ld\n", posix_memalign(&ptr, alignment, 200 * sizeof(double)), alignment);
    printf("address: %p, multiple: %f\n", ptr, ((double) (long) ptr) / alignment);
    free(ptr);
    alignment = 256;
    printf("status: %d, alignment: %ld\n", posix_memalign(&ptr, alignment, 50 * sizeof(double)), alignment);
    printf("address: %p, multiple: %f, multiple with larger alignment: %f\n", ptr, ((double) (long) ptr) / alignment, ((double) (long) ptr) / 512);
}
