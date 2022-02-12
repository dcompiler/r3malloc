CC = gcc
LFLAGS = -L . -l r3malloc
BIN = dummy
OBJECTS = dummy.o

.c.o:
	$(CC) -c $*.c

$(BIN): $(OBJECTS)
	cp ../target/debug/libr3malloc.a .
	$(CC) $(OBJECTS) $(LFLAGS) -o $(BIN)

clean:
	-rm $(BIN) $(OBJECTS) libr3malloc.a
