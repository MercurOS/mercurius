# Mercurius - Kernel for MercurOS

## Build Requirements

Install the `riscv64gc-unknown-none-elf` target by running:
```
rustup target add riscv64gc-unknown-none-elf
```

## Building

Two cargo features are provided for specifying the target hardware:

 - `qemu` for QEMU (qemu-system-riscv64)
 - `fu740` for HiFive Freedom Unmatched (SiFive fu740-c000)

To build, just run cargo build with the appropriate feature enabled:
```
$ cargo build --release --features qemu
```
or
```
$ cargo build --release --features fu740
```
