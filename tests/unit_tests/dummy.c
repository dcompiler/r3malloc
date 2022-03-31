#include <stdio.h>

// FIXME: this is a POC to see how to link Rust-compiled libraries

int test();

int main() {
	printf("Hello from C: %d\n", test());
	return 0;
}
