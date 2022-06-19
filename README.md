# prototype linked list using internal mutability

## see files `prototype_linked_list.rs` and `main.rs`

### Raison d'etre

The purpose of this branch is to mess around and test whether it is possible

to implement the linked list central to the heap structure using RefCells

instead of using raw pointers. We want this because one of the reasons for

using Rust is to make maximum use out of Rust's safety compiler checks (e.g.

to prevent concurrent access to a mutable pointer). If we use raw pointers,

it's effectively just a 1-1 translation from C++ to unsafe Rust i.e. there is

no safety gains to be made by using Rust. Hence this branch.

Because of the nature of this project (most algorithms are involved with pointer

manipulation), the decision to use raw pointers will propagate to the entire

codebase (raw pointers = unsafe Rust).

### Approach

Using `Rc<RefCell<T>>` means that we have no way of Copy-ing T (in Rust, Copy

is a direct bit-by-bit copy, so this operation would not work with Rc and RefCell,

since these need custom Clone behaviors in order for the compiler checks to work

correctly). In other words, we need to explicitly Clone the `Rc<RefCell<T>>`

"pointer" whenever we need to make a new pointer (e.g. for multiple references

in the linked list, or just while doing routine variable assignments in general).

Therefore, the approach is to replace all copy assignments of raw pointers in the

original Unsafe Rust/C++ code with explicit Clones (after replacing the raw

pointers with `Rc<RefCell<T>>`)

### (WIP) Results

Initial results do seem to show some success with this approach. See `main.rs`.

Needs more work to integrate into the actual malloc heap.

# r3malloc

To build, run

```
cargo build
```

This will create two library files: `libr3malloc.a` and `libr3malloc.so`.

To build test(s), run

```
make
```

inside of the `tests` directory.
