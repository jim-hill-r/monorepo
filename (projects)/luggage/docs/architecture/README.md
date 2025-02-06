Components

- Trolley (Client side library)
- Bellhop (Server side API)
- Closet (Backend for storing cubes, including bellhop cubes)

```mermaid
sequenceDiagram
    User->>App: Hello John, how are you?
    App-->>Trolley: Great!
    Trolley-->>Bellhop: Hoorary!
    Bellhop-->>Closet: Yaas!
    Closet-->>Bellhop: Back
```
