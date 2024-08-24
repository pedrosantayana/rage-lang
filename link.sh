#!/bin/bash

ld -o program -dynamic-linker /usr/lib/x86_64-linux-gnu/ld-linux-x86-64.so.2 /usr/lib/x86_64-linux-gnu/crt1.o /usr/lib/x86_64-linux-gnu/crti.o -lc OUTPUT.o /usr/lib/x86_64-linux-gnu/crtn.o