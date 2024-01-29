.altmacro
.macro SAVE_GP n # Macro used to save the nth general purpose register to stack
    sd x\n, \n*8(sp)
.endm
.macro LOAD_GP n  # Macro used to load the nth general purpose register to stack
    ld x\n, \n*8(sp)
.endm
    .section .text.trampoline
    .globl __alltraps
    .globl __restore
    .align 2
__alltraps:                 # function used for all trap, this address will be put in stvec
    csrrw sp, sscratch, sp  # sp -> kernel stack, sscratch -> user stack
                            # save gernal-perpose registers, no need to save x0 cause it's always zero
    sd x1, 1*8(sp)                      
    sd x3, 3*8(sp)          # skip sp(x2), we will save it later
                            # save x5~x31
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n + 1
    .endr
    csrr t0, sstatus        # we can use t0/t1/t2 freely, because they were saved on kernel stack
    csrr t1, sepc
    sd t0, 32*8(sp)         # save sstatus and sepc to TrapContext
    sd t1, 33*8(sp)
    csrr t2, sscratch       # read user stack from sscratch and save it on the kernel stack
    sd t2, 2*8(sp)
    ld t0, 34*8(sp)         # load kernel satp into t0
    ld t1, 36*8(sp)         # load trap_handler into t1
    csrw satp, t0           # switch to kernel address space
    sfence.vma
    jr t1                   # jump to trap_handler

__restore:
    # a0: *TrapContext in user address space(const), a1: user  address space token
    csrw satp, a1           # switch to user address space
    sfence.vma
    csrw sscratch, a0
    mv sp, a0
                            # restore sstatus/sepc
    ld t0, 32*8(sp)         # read sstatus
    ld t1, 33*8(sp)         # read sepc
    csrw sstatus, t0        # write sstatus
    csrw sepc, t1           # write sepc
                            # restore general purpose registers except sp/tp
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n + 1
    .endr
    ld sp, 2*8(sp)          # back to user stack
    sret                    # return 
