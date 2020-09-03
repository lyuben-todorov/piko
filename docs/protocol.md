# Protocol
The goal of this protocol is to consistently _verify_ the sequence of an unbounded stream of events. This excludes    
storage. E.g node A B and C work in a cluster, A receives some message x1. The job of A is to distribute x1 to its    
neighbours B and C and receive acknowledgements that the message has been processed, while maintaining message order. 

Each node is in either of 3 general states: `Dsc`, `WRK`, `ERR` for discovery, work and error respectively.

### Discovery phase  
A node called "Ivan" starts as DSC, looking for other nodes on the cluster. Ivan sends a `DscReq` to its pre-defined    
list of neighbours. The `DscReq` contains the sender's information. Each of the
nodes responding to a `DscReq` should send a `DscRes` with their list of neighbouring nodes and general status `DSC`, `WRK`, `ERR`.    
Each of the receivers adds the sender, Ivan, to their list. Ivan adds each of the received hosts to his list. 
### Sequence recovery phase
After discovery, Ivan can send a `SeqRes` to each of its neighbours. Each one responds with their `SEQ` number     
(which should be the same) and marks Ivan locally as `WRK`. Ivan is then promoted to `WRK`.    

The only exception is when the node's neighbour list is empty, in which case it goes straight into `WRK`.

### Work phase
A worker is required to send a heartbeat to each (for now) of its neighbours each 1s. If a node goes silent for more     
than `5`, seconds, it is removed from the cluster.

### Protocol format 
Protocol operates under TCP. A protocol 'packet' is referred to as parcel. The encoding we're using is CBOR. The parcel looks like this:     
```rust
    pub is_response: bool,
    // whether or not packet is a response
    pub parcel_type: Type,
    // type of packet
    pub id: u16,
    // id of sender node
    pub size: u16, // size of body in bytes

    pub body: Body,
``` 
Every parcel is preceded by an u8 `n` indicating the size of the protocol message in bytes, followed by `n` bytes of CBOR-serialized ProtoParcel. 

Everything past the parcel body is application-specific.

## Types
### DscReq
* Strict *Request*/Response on same TCP stream
* Body contains sender's information object
* Receiver adds information object to its state
* Expected `DscRes` in response

### DscRes
* Response to `DscReq`
* Strict Request/*Response* on same TCP stream
* Body contains a list of known neighbour's information
