#include <stdio.h>

void *malloc(long unsigned int);
void free(void *);

int main() {
	int size = 3;
	int size2 = 30000;
	int **m = (int**)malloc(size * sizeof(int*));
	for (int i = 0; i < size2; i++) {
		m[0] = (int *)malloc(sizeof(int));
		free(m[0]);
		m[1] = (int *)malloc(sizeof(int));
		free(m[1]);
		m[2] = (int *)malloc(sizeof(int));
		free(m[2]);
	}
	free(m);
}
