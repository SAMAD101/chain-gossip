# chain-gossip

## P2P GossipSub Network with Kademlia DHT

A distributed p2p network with gossipsub and kademlia DHT implemented using
rust-libp2p.

### Description

Implements a gossipsub network with kademlia DHT, and mDNS for peer discovery.
The network can have n number of nodes running locally in different terminals.
Each node can propagate messages to all other nodes in the network, and is
subscribed to a topic ("transaction"). This network implementation is to
demonstrate the use of gossipsub and kademlia DHT in a p2p network to propagating
transactions data across the network.

### Usage

- Clone the repository

```bash
git clone https://github.com/SAMAD101/chain-gossip.git
```

- Change directory to the project root

```bash
cd chain-gossip
```

- Run in multiple terminals to simulate a network of nodes

```bash
cargo run
```

New peers are automatically discovered and connected to the network using mDNS.
Terminate a node by pressing `Ctrl+c` in the terminal.

### Troubleshooting: `Publish error: InsufficientPeers`

This means the node has no connected peers. If you never see
`mDNS discovered a new peer: ...`, mDNS discovery is being blocked.
This is common on **macOS**, where the firewall or Local Network privacy
blocks mDNS multicast traffic (UDP 5353).

**Connect nodes manually (works everywhere)**

Start the first node and copy its local address:

```bash
cargo run
# Local node is listening on /ip4/127.0.0.1/udp/65360/quic-v1
```

Pass that address to every other node:

```bash
cargo run -- /ip4/127.0.0.1/udp/65360/quic-v1
```

### Demo video

[chain-gossip.webm](https://github.com/user-attachments/assets/4ac76b85-686a-4c44-af7a-cf36b3f970ee)

### Conclusion

This simple project explores the applicatgion of _libp2p_ in building decentralized
networks which can be used in context of blockchain networks.
