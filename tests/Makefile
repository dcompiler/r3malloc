CC = gcc
FLAGS = -pthread -Wl,--no-as-needed -ldl
LFLAGS = -L . -l r3malloc
CP_LIB = cp ../target/debug/libr3malloc.a .

.c.o:
	$(CC) -c $*.c

prototype: prototype.o
	$(CP_LIB)
	$(CC) $(FLAGS) prototype.o $(LFLAGS) -o prototype
