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
baybridge namespace foo # shows a mapping: $(baybridge whoami) -> bar
```

## Design

### Definitions

Each writer is identified by a keypair (_S_, _V_) where _S_ is the private signing key and _V_ is the public verifying key.
The verifying key _V_ corresponds to a keyspace _K_, the collection of names writable by _V_.
Each value in the store is addressed by a tuple (_V_, _k_), a verifying key _V_ and a name _k_.
A namespace _N_ for _k_ is the set of addresses where the name is _k_ over all verifying keys.

### Reading and writing

To write to a name in _K_, the write command must be signed by the signing key _S_.

This way, nodes do not need to trust each other.
Nodes share the signed write commands and apply the changes according to a conflict resolution strategy.
Readers can ask any node for the data and audit log of the events to derive the current state.

Values can be queried in two ways:
- By keyspace: Provide the verifying key _V_. This is the most efficient way to look up names for a given verifying key.
- By namespace: Provide the name _k_. This returns all the verifying keys known to have an entry for _k_. This is useful for discovering writers without first knowing their verifying key.

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
