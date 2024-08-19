# Bay Bridge Key-Value Store 🌉

> Synchronizing data between San Francisco and Berkeley

## Installation
```bash
nix run github:nicolaschan/baybridge
```

## Usage examples
```bash
# Start a local server
baybridge serve

# In another shell
baybridge list
baybridge set foo bar
baybridge get "$(baybridge whoami)" foo # returns bar
```

## Goals

> This is a work in progress!

- A global key-value store
- Support untrusted replicas (this is the main goal!)
- Check replication status of trusted nodes
- Subscribe to changes for realtime streaming
- Various consistency levels
- Able to build a fuse filesystem on top of it
- Swappable communication layer
- Communication through holepunched NATs
