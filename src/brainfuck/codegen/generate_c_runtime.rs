use std::io::Write;

pub fn generate_c_runtime(runtime: &mut Write, cell_type: &str, pagesize: usize) -> std::io::Result<()> {
        write!(runtime, r##"#define _GNU_SOURCE

#include <stdio.h>
#include <stdlib.h>
#include <sys/mman.h>
#include <inttypes.h>
#include <signal.h>
#include <string.h>
#include <unistd.h>

#ifndef __linux__
#   error operating system currently not supported
#endif

#define PAGESIZE {0}

{1}* mem = NULL;
size_t mem_size = 0;
"##, pagesize, cell_type)?;

        runtime.write_all(br##"
struct sigaction segv_action;

void brainfuck_main();

void memmng(int signum, siginfo_t *info, void *vctx) {
    (void)signum;

    void *ptr = info->si_addr;
    ucontext_t* ctx = (ucontext_t*)vctx;

    if (!((ptr >= (void*)mem && ptr < (void*)mem + PAGESIZE) || (ptr >= (void*)mem + (mem_size - PAGESIZE) && ptr < (void*)mem + mem_size))) {
        if (ptr >= (void*)mem + PAGESIZE && ptr < (void*)mem + (mem_size - PAGESIZE)) {
            fprintf(stderr, "pid: %d, bogus SIGSEGV at 0x%zx\n", getpid(), (uintptr_t)ptr);
            abort();
        }
        // Some other segmantation fault! This is a compiler error!
        fprintf(stderr,
            "unhandeled segmantation fault: pagesize = %zu, ptr = 0x%zX (offset %zu), mem = 0x%zX ... 0x%zX (size %zu)\n",
            (size_t)PAGESIZE,
            (uintptr_t)ptr, (uintptr_t)(ptr - (void*)mem),
            (uintptr_t)(void*)mem, (uintptr_t)((void*)mem + mem_size), mem_size);
        fflush(stderr);
        abort();
    }

    if (SIZE_MAX - PAGESIZE < mem_size) {
        fprintf(stderr, "out of address space\n");
        fflush(stderr);
        abort();
    }

    size_t new_size = mem_size + PAGESIZE;
    if (mprotect((void*)mem, PAGESIZE, PROT_READ | PROT_WRITE) != 0) {
        perror("release guard before page protection");
        abort();
    }

    if (mprotect((void*)mem + (mem_size - PAGESIZE), PAGESIZE, PROT_READ | PROT_WRITE) != 0) {
        perror("release guard after page protection");
        abort();
    }

    void *new_mem = mremap((void*)mem, mem_size, new_size, MREMAP_MAYMOVE);
    if (new_mem == MAP_FAILED) {
        perror("mremap");
        abort();
    }

    if (mprotect(new_mem, PAGESIZE, PROT_NONE) != 0) {
        perror("mprotect guard before");
        abort();
    }

    if (mprotect(new_mem + (new_size - PAGESIZE), PAGESIZE, PROT_NONE) != 0) {
        perror("mprotect guard after");
        abort();
    }

    if (ptr < (void*)mem + PAGESIZE) {
        // memory underflow, move everything to the right
        memmove(new_mem + PAGESIZE * 2, (void*)new_mem + PAGESIZE, mem_size - PAGESIZE * 2);
        ptr += PAGESIZE;
    }

    ptr = new_mem + (uintptr_t)(ptr - (void*)mem);

#ifdef __x86_64__
    ctx->uc_mcontext.gregs[REG_R12] = (intptr_t)ptr;
#else
#   error architecture currently not supported
#endif

    mem = new_mem;
    mem_size = new_size;
}

int main() {
    memset(&segv_action, 0, sizeof(struct sigaction));

    segv_action.sa_flags = SA_SIGINFO;
    segv_action.sa_sigaction = memmng;
    if (sigaction(SIGSEGV, &segv_action, NULL) == -1) {
        perror("sigaction");
        return EXIT_FAILURE;
    }

    mem_size = PAGESIZE * 3;
    mem = mmap(NULL, mem_size, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    if (mem == MAP_FAILED) {
        perror("mmap");
        return EXIT_FAILURE;
    }

    if (mprotect((void*)mem, PAGESIZE, PROT_NONE) != 0) {
        perror("mprotect guard before");
        return EXIT_FAILURE;
    }

    if (mprotect((void*)mem + (mem_size - PAGESIZE), PAGESIZE, PROT_NONE) != 0) {
        perror("mprotect guard after");
        return EXIT_FAILURE;
    }

    brainfuck_main();

    return 0;
}
"##)?;

    Ok(())
}