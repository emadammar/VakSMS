# VakSMS
A simple VakSMS's client library implemented using Rust


## Sample Usage
```rust
mod vaksms;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    vaksms::main()?;

    Ok(())
}


## Note

- Code not well tested since it's opened source
