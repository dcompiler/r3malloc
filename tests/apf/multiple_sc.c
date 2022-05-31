#include <stdio.h>
#include <stdint.h>

void *malloc(long unsigned int);
void free(void *);

int main() {
	int size = 3;
	int size2 = 30000;
	for (int i = 0; i < size2; i++) {
		int32_t **m = (int32_t**)malloc(size * sizeof(int32_t*));
		int8_t **c = (int8_t**)malloc(sizeof(int8_t*));
		free(m);
		free(c);
	}
}
