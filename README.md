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

### Demo video

<video src="chain-gossip.mkv"></video>

### Conclusion
This simple project explores the applicatgion of *libp2p* in building decentralized 
networks which can be used in context of blockchain networks.