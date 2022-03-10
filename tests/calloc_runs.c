#include <stdio.h>

void* calloc(size_t, size_t);

int main() {
    int* int_arr = (int*) calloc(10, sizeof(int));
    double* double_arr = (double*) calloc(10, sizeof(double));
    char* char_arr = (char*) calloc(10, sizeof(char));
    for (int i = 0; i < 10; i++) {
        printf("int: %d, double: %f, char: %c\n", int_arr[i], double_arr[i], char_arr[i]);
    }
}
