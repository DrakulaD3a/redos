#!/bin/sh

cargo bootimage && qemu-system-x86_64 -drive format=raw,file=target/x86_64-rudos/debug/bootimage-rudos.bin
