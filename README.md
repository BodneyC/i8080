# I8080

I got bored and decided to make a pretty bad Intel 8080 emulator.

Along the way I also got sick of writing little programs in hex... so I also wrote an assembler.

I also figured if I had an assembler, I should also have a disassemble.

## Install and Build

```sh
git clone https://github.com/BodneyC/8080-rs.git
cd 8080-rs
cargo install --path .
```

## Try it Out

There are some examples in the `rsc` directory, so, to run a hello-world:

```sh
$ i8080 run --assemble ./rsc/asm/hello-world.asm
hello world
```

Or, for a more interactive experience:

```sh
$ i8080 run --interactive --assemble ./rsc/asm/hello-world.asm
>
```

Which opens a prompt.

## Docs

Link pending

<!-- markdownlint-disable-file MD013 -->