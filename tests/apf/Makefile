CC = gcc
FLAGS = -g -O0
LFLAGS = -L . -l r3malloc
CP_LIB = cp ../../target/debug/libr3malloc.a .

SRCS = $(wildcard *.c)
OBJS := $(patsubst %.c, %.o, $(SRCS))
PROGS = $(patsubst %.o,%,$(OBJS))

all: $(PROGS)

%: %.o
	$(CP_LIB)
	$(CC) -c $(SRCS)
	$(CC) $(FLAGS) $< $(LFLAGS) -o $@ 
