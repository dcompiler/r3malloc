#include <stdio.h>

void* calloc(size_t, size_t);
void* realloc(void*, size_t);

int main() {
    double* double_arr = (double*) calloc(10, sizeof(double));

    printf("[ ");
    for (int i = 0; i < 10; i++) {
        printf("%f ", double_arr[i]);
    }
    printf("]\n");

    int new_size = 15;
    double_arr = (double*) realloc(double_arr, new_size * sizeof(double));

    printf("[ ");
    for (int i = 0; i < new_size; i++) {
        printf("%f ", double_arr[i]);
    }
    printf("]\n");

    double_arr[10] = 10; double_arr[11] = 11; double_arr[12] = 12; double_arr[13] = 13; double_arr[14] = 14;
    printf("[ ");
    for (int i = 0; i < new_size; i++) {
        printf("%f ", double_arr[i]);
    }
    printf("]\n");
}
