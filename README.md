A Brainfuck Compiler Written In Rust
====================================

Just for fun.

This compiles to Linux x86 64 assembler and emits a small C runtime for dynamic
memory management. It uses guard pages around `mmap()` allocated memory. Once a
guard is stripped (`SIGSEGV` handler) the memory is resized appropriately. It
even can handle memory underruns and resizes the memory to the bottom. It emits
assembler because after moving the memory the register holding the pointer to
the current cell needs to be adjusted and therefore I need to control what
register that is. This makes it all very architecture and operating system
dependant.

Alternatively it can also just run brainfuck programs in interpreter mode.

It supports several optimizations. If the brainfuck program doesn't depend on
input it can be executed during compilation and the resulting program will
just be a single `fwrite()` and will not contain the memory management runtime.

It calls `gcc` and `nasm` to compile the generated code.

I haven't done any x86 (64 or 32 bit) before, so that part was fun. I hope I
did it all right.

BSD License
-----------

Copyright 2019 Mathias Panzenböck

Redistribution and use in source and binary forms, with or without modification,
are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this
   list of conditions and the following disclaimer.

2. Redistributions in binary form must reproduce the above copyright notice,
   this list of conditions and the following disclaimer in the documentation
   and/or other materials provided with the distribution.

3. Neither the name of the copyright holder nor the names of its contributors
   may be used to endorse or promote products derived from this software
   without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND
ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR
ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
(INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON
ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.