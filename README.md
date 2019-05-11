# Rust CHIP-8 emulator

```
chip8 0.1.0
Rijenkii <me@rijenkii.tk>


USAGE:
    chip8 [OPTIONS] <file>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -f <freq>        clock frequency (60hz * this) [max: 255] [default: 10]

ARGS:
    <file>    ROM file
```

## Key mapping

|   |   |   |   |      |   |   |   |   |
|---|---|---|---|------|---|---|---|---|
| 1 | 2 | 3 | C | ---> | 1 | 2 | 3 | 4 |
| 4 | 5 | 6 | D | ---> | Q | W | E | R |
| 7 | 8 | 9 | E | ---> | A | S | D | F |
| A | 0 | B | F | ---> | Z | X | C | V |

## Todo (maybe)

* Custom key mapping
* Changing emulation speed
* Pausing
* Step-by-step emulation
* Debugger