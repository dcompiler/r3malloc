#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <pthread.h>

void *malloc(long unsigned int);
void free(void *);

int num_threads = 4;
pthread_t* threads;

void* do_work(void* thread_id) {
    printf("hello from %ld\n", (long) thread_id);

    double* stuff;
    for (int i = 0; i < 10; i++) {
        stuff = (double*) malloc(5 * sizeof(double));
        // printf("stuff pointer: %p\n", (void*) stuff);
        if (stuff == NULL) {
            printf("xxx> %ld error, exiting\n", (long) thread_id);
            pthread_exit(NULL);
        }

        for (int i = 0; i < 5; i++) {
            stuff[i] = 1;
        }

        int spin = 0; while (spin < 1000) spin++;  // spin

        free(stuff);
    }
    printf("---> %ld done\n", (long) thread_id);
    pthread_exit(NULL);
}

int main(int argc, char *argv[]) {
    // get num_threads input
    int c;
    while ((c = getopt (argc, argv, "t:")) != -1)
    switch (c) {
    case 't':
        num_threads = atoi (optarg);
        break;
    }

    threads = (pthread_t*) malloc(num_threads * sizeof(pthread_t));
    for (long t = 0; t < num_threads; t++) {
        if (pthread_create(&threads[t], NULL, do_work, (void*) t)) {
            printf("something went wrong creating threads");
            exit(-1);
        }
    }
    pthread_exit(NULL);
}
