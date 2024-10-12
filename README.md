# reth-exex-plugin
A Reth Execution Extension (ExEx) with shared object (.so / .dylib) plugins

`examples/minimal` directory includes a minimal dylib plugin example
that just writes [notifications](`ExExNotification`) into JSON file.

# Build
```sh
# Builds reth-exex-plugin lib and examples/minimal plugin implementation
make build
```

# Test
Note: ensure that `examples/minimal/assets/notifications.json` is exists and empty before running
```sh
make test
```
