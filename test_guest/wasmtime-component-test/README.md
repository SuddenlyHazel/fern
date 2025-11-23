
Mostly follows...

```
https://component-model.bytecodealliance.org/language-support/building-a-simple-component/rust.html
```


Build with `cargo build --target=wasm32-wasip2`


If you `cargo install wasm-tools` you can then then inspect the artifact

```
wasm-tools component wit target/wasm32-wasip2/debug/wasmtime_component_test.wasm
```

Alternatively, it appears the following VSCode plugin lets you inspect the artifact from the file file tree

```
https://marketplace.visualstudio.com/items?itemName=dtsvet.vscode-wasm
```

Wasm component package manager?
`cargo install wkg` 