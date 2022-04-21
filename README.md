# r3malloc

To build, run

```
cargo build
```

This will create two library files: `libr3malloc.a` and `libr3malloc.so`.



To build performance tests, cd into `perf_tests` and then 

```
make threadtest_test
```
By default, threadtest is testing the standard malloc.

To make threadtest with a particular allocator:

```
make threadtest_test  ALLOC=r3
```

To run threadtest

cd into `perf_tests` and then

```
./run_threadtest.sh <r>
```

The results will be written in `csv` files stored in `./data`.
