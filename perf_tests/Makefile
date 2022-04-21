SRC = ../src
#OBJ = ../obj

CXX = g++
CP_LIB = cp ../target/debug/libr3malloc.a .

WARNING_FLAGS:=-ftrapv -Wreturn-type -W -Wall \
-Wno-unused-variable -Wno-unused-but-set-variable -Wno-unused-parameter

FLAGS = -g -O0 -g -fpermissive $(WARNING_FLAGS) -fno-omit-frame-pointer #-DSHM_SIMULATING #-DDESTROY -DMEM_CONSUME_TEST

R3ALLOC_FLAGS = $(FLAGS)
BUILTIN_FLAGS = $(FLAGS)


R3_CXXFLAGS = $(R3ALLOC_FLAGS) -L. -lr3malloc
BUILTIN_CXXFLAGS = $(FLAGS)

#default malloc
CXXFLAGS = $(BUILTIN_CXXFLAGS)

ifeq ($(ALLOC),r3)
	CXXFLAGS = $(R3_CXXFLAGS)
endif

LIBS = -pthread -lstdc++ -latomic 

all: benchmark_pm


benchmark_pm: threadtest_test

threadtest_test: ./benchmark/threadtest.cpp
	$(CP_LIB)
	$(CXX) -I ./benchmark -o $@ $^ $(CXXFLAGS) $(LIBS) 

# libr3malloc.a: $(OBJECTS)
# 	ar -rcs $@ $^

clean:
	rm -f *_test
	rm -rf ../obj/*
	#rm -f libr3malloc.a
	
