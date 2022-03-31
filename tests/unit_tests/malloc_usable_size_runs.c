#include <stdio.h>

void* calloc(size_t, size_t);
size_t malloc_usable_size(void*);

int main() {
    double* small = (double*) calloc(2, sizeof(double));
    double* large = (double*) calloc(5000, sizeof(double));
    printf("large: %ld\n", malloc_usable_size(large));
    printf("small: %ld\n", malloc_usable_size(small));
}
