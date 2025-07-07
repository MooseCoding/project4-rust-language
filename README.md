This is my fourth project for the summer of making by hackclub.

We're gonna do a programming language written in Rust. I've done this priorlyin some Rust and some C but never fully so lets try it today :)

# Things to add

- [x] Function definitions & usage
- [x] Adding/subtracting
- [x] If statements
- [x] While loops
- [x] For loops
- [x] Boolean Logic
- [x] Libraries 
- [x] Arrays
- [x] Breaking
- [x] Example code for every feature 

I was going to do more, but the current structure is kinda cooked so I'll add the rest to my next programming language (which I'll write in C & have it be compiled straight to ASM). 


# Installation 

Make sure that you have rust and cargo installed, download or clone the project into a folder:

On a unix system to run as an interpreter run 
```rust 
cargo run -- examples/{file}.iron 
```

You can run any file from the examples folder and get a nice output :) 

# Basic Docs

Libraries are all in examples/lib and are noted to be 
a library file with the usage of a .steel file
Like

```
examples/lib/{library}.steel
```

The standard library of math (and more tbd) can be included 
like 

```
import <math>;
```

and using non standard libraries is like

```
import "trig";
```

The other example files should give an idea of the syntax and other features of the language. 
