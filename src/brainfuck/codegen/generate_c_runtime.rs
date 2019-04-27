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
#include <ucontext.h>

#define PAGESIZE {0}
#define CELL_T {1}
"##, pagesize, cell_type)?;

        runtime.write_all(br##"
#ifndef __linux__
#   error operating system currently not supported
#endif

volatile CELL_T* mem = NULL;
volatile size_t mem_size = 0;

struct sigaction segv_action;

void bfmain();

#ifndef NDEBUG
// this function can be called in a debugger to print information about the current state of the program
void dbg() {
    ucontext_t ctx;
    memset(&ctx, 0, sizeof(ctx));
    CELL_T *ptr;

    if (getcontext(&ctx) != 0) {
        perror("getcontext");
        ptr = NULL;
    } else {
        intptr_t rax = ctx.uc_mcontext.gregs[REG_RAX];
        intptr_t r12 = ctx.uc_mcontext.gregs[REG_R12];
        ptr = (CELL_T*)r12;
        intptr_t index = ptr - mem;
        if ((void*)ptr < (void*)mem + PAGESIZE || (void*)ptr >= (void*)mem + mem_size - PAGESIZE) {
            fprintf(stderr,
                "rax: %zd, r12: %zd, index: %zd, mem_size: %zu, *ptr: <out of bounds>\n",
                rax, r12, index - PAGESIZE, mem_size);
        } else {
            fprintf(stderr,
                "rax: %zd, r12: %zd, index: %zd, mem_size: %zu, *ptr: %zd\n",
                rax, r12, index - PAGESIZE, mem_size, (intptr_t)mem[index]);
        }
    }

    fprintf(stderr, "mem = [");
    const size_t start = PAGESIZE / sizeof(CELL_T);
    const size_t end = (mem_size - PAGESIZE) / sizeof(CELL_T);
    for (size_t i = start; i < end;) {
        CELL_T val = mem[i];

        if (i != start) {
            fprintf(stderr, ", ");
        }

        if (mem + i == ptr) {
            fprintf(stderr, ">>%d<<", val);
            ++ i;
            continue;
        }

        size_t count = 1;
        for (size_t j = i + 1; j < end && mem[j] == val && mem + j != ptr; ++ j) {
            ++ count;
        }
        if (count > 3) {
            fprintf(stderr, "%zd... x%zu", (intptr_t)val, count);
            i += count;
        } else {
            fprintf(stderr, "%zd", (intptr_t)val);
            ++ i;
        }
    }
    fprintf(stderr, "]\n");
}
#endif

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
        abort();
    }

    if (SIZE_MAX - PAGESIZE < mem_size) {
        fprintf(stderr, "out of address space\n");
        abort();
    }

    const size_t new_size = mem_size + PAGESIZE;
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
        memmove(new_mem + PAGESIZE * 2, new_mem + PAGESIZE, mem_size - PAGESIZE * 2);
        memset(new_mem + PAGESIZE, 0, PAGESIZE);
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

    bfmain();

    return 0;
}
"##)?;

    Ok(())
}