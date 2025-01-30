use libp2p::{
    futures::StreamExt,
    gossipsub,
    mdns,
    kad::{self, store::MemoryStore as TStore, Record, RecordKey, Quorum},
    swarm::{SwarmEvent, NetworkBehaviour},
    Multiaddr,
};
use std::{
    collections::hash_map::DefaultHasher,
    error::Error,
    hash::{Hash, Hasher},
    time::{SystemTime, Duration},
};
use tokio::{io, io::AsyncBufReadExt, select};
use serde::{Serialize, Deserialize};

// Create a custom behaviour for the network
#[derive(NetworkBehaviour)]
struct MyBehaviour {
    gossipsub: gossipsub::Behaviour,
    mdns: mdns::tokio::Behaviour,
    kademlia: kad::Behaviour<TStore>,
}

// Struct to handle transaction messages
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TransactionMessage {
    signature: String,
    sender: String,
    timestamp: u64,
    tx_data: Vec<u8>,
}

fn create_dummy_transaction(sender: String) -> TransactionMessage {
    // Function to create dummy transaction
    TransactionMessage {
        signature: format!("signature-{}", rand::random::<u64>()),
        sender,
        timestamp: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        tx_data: vec![0, 1, 2, 3],
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_quic()
        .with_behaviour(|key| {
            // To content-address message, we can take the hash of message and use it as an ID.
            let message_id_fn = |message: &libp2p::gossipsub::Message| {
                let mut s = DefaultHasher::new();
                message.data.hash(&mut s);
                libp2p::gossipsub::MessageId::from(s.finish().to_string())
            };

            // Create a gossipsub config
            let gossipsub_config = gossipsub::ConfigBuilder::default()
                .heartbeat_interval(Duration::from_secs(5))
                .validation_mode(gossipsub::ValidationMode::Strict)
                .message_id_fn(message_id_fn)
                .build()
                .map_err(|msg| io::Error::new(io::ErrorKind::Other, msg))?;

            // Build a gossipsub network behaviour
            let gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(key.clone()),
                gossipsub_config,
            )?;

            // Build a kademlia network behaviour
            let kademlia = kad::Behaviour::new(
                key.public().to_peer_id(),
                TStore::new(key.public().to_peer_id())
            );

            // Build a mdns network behaviour
            let mdns = mdns::tokio::Behaviour::new(
                mdns::Config::default(),
                key.public().to_peer_id()
            )?;

            Ok(MyBehaviour { gossipsub, mdns, kademlia })
        })?
        .build();

    // Subscribe to a topic
    let topic = gossipsub::IdentTopic::new("transaction");
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

    // Listen on a local address using QUIC (UDP)
    let addr: Multiaddr = "/ip4/0.0.0.0/udp/0/quic-v1".parse().unwrap();
    swarm.listen_on(addr)?;

    println!("Node is listening for connections...");

    // Read full lines from stdin
    let mut stdin = io::BufReader::new(io::stdin()).lines();

    // Get the local peer ID
    let mut local_peer_id = swarm.local_peer_id().to_string();

    // Handle swarm events
    loop {
        select! {
            Ok(Some(line)) = stdin.next_line() => {
                // Create a dummy transaction
                let transaction = create_dummy_transaction(local_peer_id.clone());

                // Serialize the transaction
                let serialized_transaction = bincode::serialize(&transaction).unwrap();

                // Publish the serialized transaction to the GossipSub network
                if let Err(e) = swarm
                    .behaviour_mut().gossipsub
                    .publish(topic.clone(), serialized_transaction) {
                    println!("Publish error: {e:?}");
                } else {
                    println!("Published transaction: {:?}", transaction);
                }
            }
            event = swarm.select_next_some() => match event {
                SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, _multiaddr) in list {
                        println!("mDNS discovered a new peer: {peer_id}");
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                    }
                },
                SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                    for (peer_id, _multiaddr) in list {
                        println!("mDNS discover peer has expired: {peer_id}");
                        swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                    }
                },
                SwarmEvent::Behaviour(MyBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                    propagation_source: peer_id,
                    message_id: id,
                    message,
                })) => {
                    println!(
                        "Got message: '{}' with id: {id} from peer: {peer_id}",
                        String::from_utf8_lossy(&message.data),
                    );

                    // Deserialize the transaction
                    let transaction: TransactionMessage = bincode::deserialize(&message.data).unwrap();
                    println!(
                        "Received transaction: {:?} with id: {id} from peer: {peer_id}",
                        transaction,
                    );

                    // Store the transaction in Kademlia
                    let transaction_key = RecordKey::new(&format!("transaction: {}", id));
                    let transaction_record = Record {
                        key: transaction_key,
                        value: message.data.clone(),
                        publisher: Some(peer_id),
                        expires: None,
                    };
                    swarm.behaviour_mut().kademlia.put_record(transaction_record, Quorum::One)?;

                    // Store the peer ID in Kademlia
                    let peer_key = format!("peer:{}", peer_id);
                    let peer_key = RecordKey::new(&format!("peer:{}", peer_id));
                    let peer_record = Record {
                        key: peer_key,
                        value: peer_id.to_bytes(),
                        publisher: Some(peer_id),
                        expires: None,
                    };
                    swarm.behaviour_mut().kademlia.put_record(peer_record, Quorum::One)?;
                },
                SwarmEvent::Behaviour(MyBehaviourEvent::Kademlia(kad::Event::RoutingUpdated {
                    peer,
                    ..
                })) => {
                    println!("Routing table updated, peer joined DHT: {peer}");
                },
                SwarmEvent::Behaviour(MyBehaviourEvent::Kademlia(kad::Event::OutboundQueryProgressed{
                    result,
                    ..
                })) => {
                    println!("Outbound query progressed!");
                },
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("Local node is listening on {address}");
                },
                _ => {}
            }
        }
    }
}
