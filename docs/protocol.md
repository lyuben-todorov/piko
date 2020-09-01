# Protocol
The goal of this protocol is to consistently _verify_ the sequence of an unbounded stream of events. This excludes    
storage. E.g node A B and C work in a cluster, A receives some message x1. The job of A is to distribute x1 to its    
neighbours B and C and receive acknowledgements that the message has been processed, while maintaining message order. 
For example, if the cluster reaches a consensus on the last message processed having sequence number `5`, a node is    
capable of acquiring a sequence 'lock' on the next sequence number, `6`, and publishing it.    

Each node is in either of 3 general states: `DSC`, `WRK`, `ERR` for discovery, work and error respectively.

### Discovery phase  
A node called "Ivan" starts as DSC, looking for other nodes on the cluster. Ivan sends a `DscReq` to its pre-defined    
list of neighbours. The `DscReq` contains the sender's information. Each of the
nodes responding to a `DscReq` should send a `DscRes` with their list of neighbouring nodes and general status `DSC`, `WRK`, `ERR`.    
Each of the receivers adds the sender, Ivan, to their list. Ivan adds each of the received hosts to his list. After     
this operation, Ivan can send a `SeqRes` to each of its neighbours. Each one responds with their `SEQ` number     
(which should be the same) and marks Ivan locally as `WRK`. Ivan is then promoted to `WRK`.    

The only exception is when the node's neighbour list is empty, in which case it goes straight into `WRK`.

### Work phase
A worker is required to send a heartbeat to each (for now) of its neighbours each 1s. If a node goes silent for more     
than `5`, seconds, it is removed from the cluster.

### Protocol format 
Protocol operates under TCP. The protocol wraps the 'message' within a `parcel`. The parcel contains a 64 bit header:     
```rust
    pub id: u16
    pub size: u16,
    pub meta: u16,
    pub parcel_type: Type, // u16
``` 
The size field indicates how many bytes _after_ the header are still protocol-specific. This is the body of the parcel,
which contains the 'arguments' for different types of messages our protocol supports.


Everything past the parcel body is application-specific.

## Types
### DscReq
* Strict *Request*/Response on same TCP stream
* Body contains sender's information object
* Receiver adds information object to its state
* Expected `DscRes` in response

### DscRes
* Response to *DscReq*
* Strict Request/*Response* on same TCP stream
* Body contains a list of known neighbour's information
