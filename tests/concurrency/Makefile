CC = gcc
FLAGS = -pthread
LFLAGS = -L . -l r3malloc
CP_LIB = cp ../../target/debug/libr3malloc.a .

.c.o:
	$(CC) -c $*.c

same_size_class: same_size_class.o
	$(CP_LIB)
	$(CC) $(FLAGS) same_size_class.o $(LFLAGS) -o same_size_class
