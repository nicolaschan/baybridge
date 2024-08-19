# Bay Bridge Key-Value Store 🌉

```bash
bay get bay://bay.example.com/some-key
bay get bay://bay.example.com/some-key --consistency=strong

bay sub bay://bay.example.com/some-key

bay set some-key value
bay set --keyfile=some-key some-key value

bay replica bay://bay.example.com/some-key/*
```

## Goals

- A global key-value store
- Support untrusted replicas (this is the main goal!)
- Check replication status of trusted nodes
- Subscribe to changes for realtime streaming
- Various consistency levels
- Able to build a fuse filesystem on top of it
- Swappable communication layer
- Communication through holepunched NATs
