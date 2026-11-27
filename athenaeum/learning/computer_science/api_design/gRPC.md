# gRPC

gRPC is a remote procedure call framework. You describe services and
messages in a `.proto` file, generate client and server code from it, and
send Protocol Buffers (a compact binary format) over HTTP/2.

- **Call types:** unary (one request, one response), server streaming,
  client streaming and bidirectional streaming.
- **Good for:** service-to-service calls where both ends are yours, with a
  typed contract and streaming.
- **Less good for:** browsers, which can't speak gRPC directly and need
  gRPC-Web or Connect through a proxy; public APIs, where JSON over HTTP is
  easier to call and debug.
- **Contracts evolve by field number:** add fields, never reuse or renumber
  them, and mark removed ones `reserved`.
- **In Rust:** `tonic` for client and server, `prost` for the generated
  messages.

## Sources

- [Introduction to gRPC](https://grpc.io/docs/what-is-grpc/introduction/)
- [Protocol Buffers](https://protobuf.dev/)
- [Google API design guide](https://cloud.google.com/apis/design) (resource-oriented design for gRPC and REST)
- [tonic](https://github.com/hyperium/tonic)
- [Connect](https://connectrpc.com/) (gRPC-compatible, works from browsers)
