.altmacro
.macro SAVE_GP n # Macro used to save the nth general purpose register to stack
    sd x\n, \n*8(sp)
.endm
.macro LOAD_GP n  # Macro used to load the nth general purpose register to stack
    ld x\n, \n*8(sp)
.endm
    .section .text
    .globl __alltraps
    .globl __restore
    .align 2
__alltraps:                 # function used for all trap, this address will be put in stvec
    csrrw sp, sscratch, sp  # sp -> kernel stack, sscratch -> user stack
    addi sp, sp, -34*8      # allocate a TrapContext on kernel stack
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
    mv a0, sp               # set input argument of trap_handler(cx: &mut TrapContext)
    call trap_handler

__restore:
    # case1: restore running app by __restore
    # case2: back to U after handling trap
    # this function will be used to restore the context
    mv sp, a0               # sp -> kernel stack(after allocated), sscratch -> user stack
                            # restore sstatus/sepc
    ld t0, 32*8(sp)         # read sstatus
    ld t1, 33*8(sp)         # read sepc
    ld t2, 2*8(sp)          # read sscratch(user stack pointer)
    csrw sstatus, t0        # write sstatus
    csrw sepc, t1           # write sepc
    csrw sscratch, t2       # write sscratch
                            # restore general purpose registers except sp/tp
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n + 1
    .endr
    addi sp, sp, 34*8       # release TrapContext on kernel stack
    csrrw sp, sscratch, sp  # kernel stack -> sscratch, user stack -> sp
    sret                    # return 
