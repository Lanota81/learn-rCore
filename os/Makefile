build: 
	@cargo build --release

run: run-defined

BOOTLOADER := /root/rCore-Tutorial-v3/bootloader/rustsbi-qemu.bin

KERNEL_RAW := target/riscv64gc-unknown-none-elf/release/os
KERNEL_BIN := target/riscv64gc-unknown-none-elf/release/os.bin

KERNEL_ENTRY_PA := 0x80200000

QEMU_ARGS := -machine virt \
		     -nographic \
		     -bios $(BOOTLOADER) \
		     -device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA)

copy-bin: 
	@rust-objcopy --strip-all $(KERNEL_RAW) -O binary $(KERNEL_BIN)

pre: build \
	copy-bin

run-defined: pre
	@qemu-system-riscv64 $(QEMU_ARGS)

gdbserver: pre
	@qemu-system-riscv64 $(QEMU_ARGS) -s -S

GDB_ARGS := -ex 'file $(KERNEL_RAW)' \
			-ex 'set arch riscv:rv64' \
			-ex 'target remote localhost:1234'

gdbclient: 
	@riscv64-unknown-elf-gdb $(GDB_ARGS)