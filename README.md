# I8080

I got bored and decided to make a pretty bad Intel 8080 emulator.

Along the way I got sick of writing little programs in hex... so I also wrote an assembler.

Then I figured with an assembler should come a disassembler.

## Install and Build

```sh
git clone https://github.com/BodneyC/i8080.git
cd i8080
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

See the [docs](https://bodneyc.github.io/i8080) for more.

<!-- markdownlint-disable-file MD013 -->
