#include <stdio.h>
#include <stdlib.h>
#include <pthread.h>
#include <unistd.h>

void add_to_linked_list(int);
int remove_from_linked_list();
void print_linked_list();

int num_threads = 8;
pthread_t* threads;

void* do_work(void* thread_id) {
    printf("hello from %ld\n", (long) thread_id);

    for (int i = (int) thread_id; i < 100 * (int) thread_id; i++) {
        add_to_linked_list(i);
    }

    // for (int i = (int) thread_id; i < 100 * (int) thread_id; i++) {
    //     remove_from_linked_list();
    // }
    printf("---> %ld done\n", (long) thread_id);
    pthread_exit(NULL);
}

int main(int argc, char* argv[]) {
    // get num_threads input
    char * optarg;
    int c;
    while ((c = getopt (argc, argv, "t:")) != -1)
    switch (c) {
    case 't':
        num_threads = atoi (optarg);
        break;
    }

    // initial test
    add_to_linked_list(5);
    print_linked_list();
    printf("removed: %d\n", remove_from_linked_list());
    print_linked_list();

    threads = (pthread_t*) malloc(num_threads * sizeof(pthread_t));
    for (long t = 0; t < num_threads; t++) {
        if (pthread_create(&threads[t], NULL, do_work, (void*) t)) {
            printf("something went wrong creating threads");
            exit(-1);
        }
    }
    pthread_exit(NULL);
}
