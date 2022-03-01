#include <stdio.h>

void *malloc(long unsigned int);

int main() {
	int *arr = (int*)malloc(10 * sizeof(int));
	int *arr2 = (int*)malloc(10 * sizeof(int));
	for (int i = 0; i < 10; i++) {
		arr[i] = i + 1;
		arr2[i] = i - 1;
	}
	for (int i = 0; i < 10; i++) {
		printf("%d %d\n", arr[i], arr2[i]);
	}
}
