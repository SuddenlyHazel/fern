# Generate a new plugin from the schema
```
xtp plugin init --schema-file ../fern-runtime/fern-schema.yaml
```

# Validate changes to the schema 
```
xtp schema validate ../fern-runtime/fern-schema.yaml
```

# Build the plugin and produce a wasm artifact
```
xtp plugin build
```