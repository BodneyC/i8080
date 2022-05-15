        LXI H, _hello ; Load HL with the starting address of 'hello world'
_do:    MOV A, M      ; Move the char into A
        OUT 0         ; Output A to out device 0 (console)
        CPI 0x00      ; Compare with 0
        JZ _done      ; Jump if 0
        INX H         ; Increment HL
        JMP _do       ; Loop
_done:  HLT           ; Halt
_hello: DB 'hello world', 0x00
