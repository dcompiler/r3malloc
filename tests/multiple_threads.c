#include <stdio.h>
#include <pthread.h>

void *malloc(long unsigned int);
void free(void *);

void *test_thread(void *tid_) {
	long tid = (long)tid_;

	int *arr = (int*)malloc(5 * sizeof(int));
	int *arr2 = (int*)malloc(5 * sizeof(int));
	for (int i = 0; i < 5; i++) {
		arr[i] = i + 1;
		arr2[i] = i - 1;
	}
	free(arr);
	free(arr2);

	pthread_exit(NULL);
}

int main() {
	void *status;
	int nthreads = 1;

	pthread_t *threads = (pthread_t *)malloc(nthreads * sizeof(pthread_t));
	for (long i = 0; i < nthreads; i++)
		pthread_create(&threads[i], NULL, test_thread, (void *)i);
	for (long i = 0; i < nthreads; i++)
		pthread_join(threads[i], &status);
	free(threads);
}
