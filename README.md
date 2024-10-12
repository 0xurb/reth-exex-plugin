# reth-exex-plugin
A Reth Execution Extension (ExEx) with shared object (.so / .dylib) plugins

`examples/minimal` directory includes a minimal dylib plugin example
that just writes [notifications](`ExExNotification`) into JSON file.

# Build
Note: See `Makefile` to run for `examples/minimal` or `reth-exex-plugin` lib separately.
```sh
# Builds `reth-exex-plugin` lib and `examples/minimal` plugin implementation.
# By default in release mode.
# Note: `examples/minimal` should be build in release mode to produce .dylib file
make build
# Linting and formatting.
make fix
```

# Test
Note: ensure that `examples/minimal/assets/notifications.json` is exists and empty before running

```sh
make test
```
