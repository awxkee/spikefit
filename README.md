# spikefit

Peak detection in Rust.

## Features

### Design a filter and apply it
```rust
let peaks = FindPeaksOptions::new(&signal).compute()?;

// Full filter chain:
let peaks = FindPeaksBuilder::new(&signal)
  .height(3.0)
  .threshold(0.5)
  .distance(4)
  .prominence(1.0)
  .width(2.0)
  .find()?;
```

This project is licensed under either of

- BSD-3-Clause License (see [LICENSE](LICENSE.md))
- Apache License, Version 2.0 (see [LICENSE](LICENSE-APACHE.md))

at your option.
