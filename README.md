risclient
=========

A very simple Rust crate to stream messages from the [RIS Live](https://ris-live.ripe.net/) service provided by RIPE.

Under the hood, this uses tungstenite and tokio to stream messages, deserialising them with serde from JSON.
The `stream*` methods create a tokio task underneath the hood to keep deserialising and sending messages in the background.
A Receiver is returned from the `stream*` methods so you can asynchronously iterate over the stream.

If you find this useful, let me know! If you make money using it, good for you.

TODO
====
 - error handling could use improvement
 - potentially moving to a `yield` pattern could be more flexible
 - some tests, probably

author
======
James "ec0" Hebden <ec0@tachibana.systems>
