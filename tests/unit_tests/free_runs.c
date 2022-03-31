#include <stdio.h>

void *malloc(long unsigned int);
void free(void *);

int main() {
	int *arr = (int*)malloc(10 * sizeof(int));
	int *arr2 = (int*)malloc(10 * sizeof(int));
	char *arr3 = (char*)malloc(20);
	free(arr);
	free(arr2);
	free(arr3);
}
