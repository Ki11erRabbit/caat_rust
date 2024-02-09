# CAAT for Rust

Commands as Arrow Types for Rust

CAAT provides a way for programs to call each other regardless of the language used as if they were functions. This is the library for the Rust language, it provides the required functions, structs, and macros for making programs that can call foreign code.

## How it works

When you call a foreign command, an environment variable called `CAAT_ARGS` is set with a JSON string that represents the arguments passed into the function. They are also passed in via the command line arguments but this is for legacy reasons. The JSON string gets parsed into a format that the language can understand. At the same time, the caller opens up a socket at `/tmp/caat_pid.sock` where pid is the pid of the caller process. This is set as the `CAAT_SOCKET` variable which is also passed into the callee. When the callee is done with what it is doing, it should call the function or macro that will write the return value back to the caller. This will also end the program.


## Example
#### Program 1
```rust
use caat;

fn main() {
    let ff = caat::ForeignFunction::new("other");

    let return_value = ff.call(&[caat::Value(String::from("Hello, World!"))]);

    println!("{}", return_value);
}
```
#### Program 2
```rust
use caat;

fn main() {
    let args = caat.args();
    println!("{}", args.next().unwrap());

    return_caat!("Successful print!");
}
```
#### Output
```
./program_1
Hello, World!
Successful print!
```

