# Protocol
The goal of this protocol is to consistently _verify_ the sequence of an unbounded stream of events. This excludes    
storage. The cluster relies on each node having a synchronized clock. 

Each node is keeping track of a local a min-heap priority queue of 'resource locks'. The top of the priority queue gives 
resource ownership to the node who requested it. The node with resource ownership must then release it.

### ResourceLock
Client pushes message to node `A`. `A` sends its resource lock to every neighbouring node. 
Upon receiving a resource lock, each node inserts it into its resource queue and acknowledges the request.
Upon receiving all acknowledges, `A` inserts the lock into its own resource queue.  

### ResourceRelease
Each node should always operate on the same `root` resource lock of the priority queue. 

### Resolving concurrent requests
Whenever a node is in the process of acknowledging a Resource Request, it locks its `f_access` mutex. Any other request
or outgoing request attempts first tries to acquire `f_access` lock before entering its critical section.  

### What shouldn't happen
* Node A receives ResourceRequest for message `a` at `t` and dispatches its acknowledgement at a later time `t'`. 
*Between* `t` and `t'` a local client tries to send a message `b` to the cluster.    

This must be interpreted as `a -> b`.

* Node B sends a ResourceRequest for message `a` to Node A with timestamp `t0`. The request arrives at A at time `t`
and is acknowledged at time `t'`. Before `t`, a local client to A tries sending a message `b` with timestamp `t1` where
`t0 -> t1` to the cluster.

This must be interpreted as `a -> b`.
### Discovery phase  
A node starts as `Dsc`, looking for other nodes on the cluster. It sends a `DscReq` to its pre-defined 
list of neighbours. The `DscReq` contains the sender's information. Each of the nodes responding to a `DscReq` 
should send a `DscRes` with their list of neighbouring nodes and general status. Each of the receivers adds the sender 
to their list. The sender adds each of the received hosts to his list. The only exception is when the starting 
node's neighbour list is empty, in which case it goes straight into `Wrk`.

### Sequence recovery
After discovery, a node can send a `SeqRes` to each of its neighbours. Each one responds with their `SEQ` number.

### Work phase
A worker keeps track of its neighbours by sending a heartbeat to each of its working neighbours on a specified timeout. 
If a node goes silent for more a number of repeated unacknowledged pings, it is removed from the cluster.

### State change
A node can send a `StateChange` containing its new state so that its neighbours can locally update it.

### External address query
A node can send a `ExtAddrReq` to a neighbour to figure out the external address it's being contacted on. 

### Protocol format 
Protocol operates under TCP. A protocol 'packet' is referred to as parcel. The encoding we're using is CBOR. 
The parcel looks like this:     
```rust
pub struct ProtoParcel {
    // whether or not packet is a response
    pub is_response: bool,
    // type of packet
    pub parcel_type: Type,
    // id of sender node
    pub id: u16,
    // message body
    pub body: Body,
}
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
* Strict Request/*Response* on same TCP stream
* Response to `DscReq`
* Body contains a list of known neighbour's information

### SeqRes
* Strict *Request*/Response on same TCP stream
* Empty body 

```
Todo:

Event registration and propagation.

Shared state: clients.
Shared state: neighbours.

Receive message from neighbour on network thread -> ack -> dispatch thread to tell clients
Receive message from client on main thread -> propagate to neighbours -> receive acks -> dispatch thread to tell clients

Figure out http/application interface.

Sequencing.

```

### References:
> [Time, Clocks and the Ordering of Events in a Distributed System][1]

[1]: https://www.microsoft.com/en-us/research/publication/time-clocks-ordering-events-distributed-system/?from=http%3A%2F%2Fresearch.microsoft.com%2Fen-us%2Fum%2Fpeople%2Flamport%2Fpubs%2Ftime-clocks.pdf
