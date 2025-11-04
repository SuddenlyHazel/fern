# Fern 🌿
Maybe your fav weird distributed wasm stack

<img src="assets/fern.png" alt="Fern Logo" width="200">

## What Makes Fern Weird? 🤔

- **🗄️ SQLite Support**: Enhanced type hints, rich metadata, and query plans all ready for guests to leverage
- **🔑 Key-Value Storage**: For stashing some JSON and getting on with your development
- **📡 Gossip Networking**: Your plugins can now chat with each other across the internet (what will they talk about!?)

## Fern + [Iroh](https://github.com/n0-computer/iroh) 🦀

<img src="assets/FernAndIroh.png" alt="Fern and Iroh" width="200">

Fern uses the awesome Iroh stack to provide p2p networking features to guests that "Just Works"™. Because why should networking be hard?

## Fern + [Extism](https://github.com/extism/extism)

Fern uses the extremely convenient Extism stack to handle all the nitty-gritty wasm side of things. As an added bonus, this means we get to leverage [XTP](https://www.getxtp.com/) which provides plugin template generation (with type hinting for all host functions) and a CLI for building modules.

## Getting Your Hands Dirty 🚀
> NOTE: You can write plugins but we don't have a way to actually "deploy" them just yet. You could clone the repo and manually connect things up if you wanted.

Interested in creating your own weird and wonderful Fern plugin? It's easier than explaining why this runtime exists:

```bash
xtp plugin init --schema-file fern-runtime/fern-schema.yaml
```

## Project Status 📋

Still very much in a "proof of concept" stage. There is a lot of work to be done! 

