# rust_os_WIP
Minimal Operating System for x86_64-unknown-none targets (WIP)

```ps1
cargo bootimage
```
```ps1
qemu-system-x86_64 -drive format=raw,file=./target/x86_64-rust_os/debug/bootimage-rust_os.bin
```

