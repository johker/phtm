## core

- Run ``` cargo run -- "$(<src/core/spatial_pooler.push)"``` to start the module passing the spatial_pooler.
- Run ``` cargo test``` to run unit tests. 

To build with a local pushr version replace the pushr dependency in 'Cargo.toml': 
```pushr = { path = "../../pushr", version = "0.4.0" }````

## ideas

- Encoders: text (semantic folding), video, simulation (motor)
- More layers
- Simplify instruction set to facilitate variation

