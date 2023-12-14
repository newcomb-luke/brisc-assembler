# brisc-assembler

An assembler for the BRISC assembly language.

## Build Instructions

With Rust installed, simply use cargo to build the project:

```bash
cd brisc-assembler/
cargo build --release
```

## Usage Instructions

This will assemble the file **prog.basm** into **prog.bin**:

```bash
brisc-assembler prog.basm
```

These will both assemble the file **prog.basm** into **output.bin**:

```bash
brisc-assembler prog.basm -o output.bin

brisc-assembler prog.basm --output-path output.bin
```

Display help:

```bash
brisc-assembler --help
```

## Language Reference

### Notation

Any Register - `rX`  
Label - `<label>`  
Integer - `<integer>`  

### Math Instructions

#### Add
```
add rX, rx
```
#### Subtract
```
sub rX, rX
```
#### Bitwise And
```
and rX, rX
```
#### Bitwise Or
```
or rX, rX
```
#### Bitwise Exclusive Or
```
xor rX, rX
```
#### Bitwise Invert
```
inv rX
```
#### Bitwise Shift Right
```
sr rX, rX
```
#### Bitwise Shift Left
```
sl rX, rX
```

### Memory Instructions

#### Load Immediate Value
```
ldi rX, <integer>
```

### I/O Instructions

#### Input from Source
```
in rX, <integer>
```

#### Output to Sink
```
out rX, <integer>
```

#### Sources (Input)

| Name     | Value |
|----------|-------|
| Switches | 0     |
| BTNC     | 1     |
| BTNU     | 2     |
| BTNL     | 3     |
| BTNR     | 4     |
| BTND     | 5     |
| COUNTER  | 6     |

#### Sinks (Output)

| Name            | Value |
|-----------------|-------|
| 7 Segment Right | 0     |
| 7 Segment Left  | 1     |

### Jump Instructions

#### Jump if Zero
```
jz rX, <integer>
jz rX, <label>
```

#### Jump if Less Than
```
jlt rX, <integer>
jlt rX, <label>
```

#### Jump Always
```
j <integer>
j <label>
```

### Labels

For loop example:

```
ldi r0, 0 ; Our counter 'i'
ldi r1, 1 ; A register just to hold our increment, 1

for_loop:
    ldi r2, 5           ; Our maximum value, 5

    ; Do something here you want to happen 5 times

    sub r2, r0          ; r2 = r2 - r0
    jz r2, for_loop_end ; If (5 - i) == 0, break the loop
    add r0, r1          ; i++
    j for_loop          ; Loop again
foor_loop_end:
    nop                 ; Continue with the rest of the program
```