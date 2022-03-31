CC = gcc
FLAGS =
LFLAGS = -L . -l r3malloc
CP_LIB = cp ../../target/debug/libr3malloc.a .

.c.o:
	$(CC) -c $*.c

free: free_runs.o
	$(CP_LIB)
	$(CC) $(FLAGS) free_runs.o $(LFLAGS) -o free_runs

malloc: malloc_runs.o
	$(CP_LIB)
	$(CC) $(FLAGS) malloc_runs.o $(LFLAGS) -o malloc_runs

calloc: calloc_runs.o
	$(CP_LIB)
	$(CC) $(FLAGS) calloc_runs.o $(LFLAGS) -o calloc_runs

realloc: realloc_runs.o
	$(CP_LIB)
	$(CC) $(FLAGS) realloc_runs.o $(LFLAGS) -o realloc_runs

malloc_usable_size: malloc_usable_size_runs.o
	$(CP_LIB)
	$(CC) $(FLAGS) malloc_usable_size_runs.o $(LFLAGS) -o malloc_usable_size_runs

posix_memalign: posix_memalign_runs.o
	$(CP_LIB)
	$(CC) $(FLAGS) posix_memalign_runs.o $(LFLAGS) -o posix_memalign_runs

dummy: dummy.o
	$(CP_LIB)
	$(CC) $(FLAGS) dummy.o $(LFLAGS) -o dummy
