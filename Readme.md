# Line density

Compute a line density with normalization to accurately show density in time series data.

## Run

You can pass the number of time series that should be generated as a command line argument. 

### In development mode

`cargo run`

### In release mode

```
cargo build --relase
target/release/line-density
```

To run an experiment with one million time series, run `target/release/line-density 100000`.

## Performance

The current implementation does not use GPUs but it runs the density computation parallel. On a machine with 120 cores, the computation for one million time series takes about 16 seconds.
