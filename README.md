# Intruducer
> The intruder introducer!

A [Rust](https://www.rust-lang.org/) crate to load a shared library into a target process without using `ptrace`. This is a portable rewrite of [dlinject](https://github.com/DavidBuchanan314/dlinject).

![example](https://user-images.githubusercontent.com/46219656/146436105-b4f29bd0-e98b-498b-b75c-5ce3680974da.gif)

## Compatibility
It should work for `x86`, `x86-64`, `arm` and `aarch64`, for both Linux and Android.

## Example
```sh
# Build binary
cargo build --example intruducer
# Build victim
cargo build --example victim
# Build library
rustc ./examples/evil.rs --crate-type cdylib --out-dir ./target/debug/examples

# Execute the victim
cd ./target/debug/examples
./victim

# Within a new shell
cd ./target/debug/examples
./intruducer -l ./libevil.so `pidof victim`
```

## How it works
1) Retrieve the instruction pointer (`ip`) of the target process reading `/proc/<pid>/syscall`;
2) Open `/proc/<pid>/mem` and backs up the content at `ip`;
3) Generate the two shellcodes, and saves the last one to a file.
4) Write the first shellcode to the target process memory at `ip` - the execution flow is now altered.
5) The first shellcode loads and executes the second shellcode.
6) The second shellcode restores the original code, calls `dlopen` and branches to `ip` - the original execution flow is resumed.

## Caveats
- It makes large applications crash when a lot of computing is going on - this happens when a thread is executing the first shellcode and another one is executing the second shellcode, which restores the original code. A possible solution consists in freezing every thread but one using `/sys/fs/cgroup/freezer`, let this one perform the whole task and then thawing all the others. However, this only seemed to reduce the chance of crashes.
- A register (`x28`) will be clobbered on `aarch64` - I found no way to branch to an absolute virtual address without using a register.
- When targeting an Android application, both library and second shellcode binary blob will be copied to its native library directory - changing the security context to `u:object_r:apk_data_file:s0` is not enough for the library file.