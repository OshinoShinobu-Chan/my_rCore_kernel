.altmacro
.macro SAVE_SN n
    sd s\n, (\n+2)*8(a0)
.endm
.macro LOAD_SN n
    ld s\n, (\n+2)*8(a1)
.endm
    .section .text
    .globl __switch
__switch:
    # __switch(
    #     current_task_cx_ptr: *mut TaskContext, // in a0
    #     next_task_cx_ptr: *const TaskContext   // in a1
    # )
    # Step 1
    # save kernel stack of current task
    sd sp, 8(a0)
    # Step 2
    # save ra and s0~s11 of current execution environment
    sd ra, 0(a0)
    .set n, 0
    .rept 12
        SAVE_SN %n
        .set n, n + 1
    .endr
    # Step 3
    # restore ra and s0~s11 of next execution environment
    ld ra, 0(a1)
    .set n, 0
    .rept 12
        LOAD_SN %n
        .set n, n + 1
    .endr
    # Step 4
    # restore kernel stack of next task
    ld sp, 8(a1)
    ret