# W10k Challenge (Rust)



This is a simple web server to see if we can handle 10k concurrent websocket connections with different project 
requirements.

## Broadcast

`broadcast` implements a server that would need to send messages to all clients as well as do some processing
on messages that the clients send. This type of websocket server would not implement any messages to a specific client
either from the server or from other clients. The functionality of this server would be to:

* Process (print) messages it receives
* Broadcast some data (the current time) to all websockets at a pre-defined (by `PING_INTERVAL`) interval 

## Client-to-Client

`client2client` implements a server that enables communication between two clients using websockets. We can imagine
that this could be some kind of messaging service using end-to-end encryption, where the server just facilitates the 
message passing. This server would need to:

* Read a message from a client and pass it to another client

[k6](https://k6.io/docs/) is a good tool for load testing servers with virtual users. See
[w10k-k6-clients](https://github.com/david-wiles/w10k-k6-clients) for the test files.

This implementation is based on the [warp "websockets_chat" example](https://github.com/seanmonstar/warp/blob/master/examples/websockets_chat.rs).
