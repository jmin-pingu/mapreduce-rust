# Paper Replication: MapReduce

My personal implementation of [MapReduce](https://research.google/pubs/mapreduce-simplified-data-processing-on-large-clusters/) in Rust.

**Goals**
- Reimplement MapReduce in Rust.
- Allow user-supplied programs that follow the MapReduce API to work at runtime.
- Get comfortable with Rust RPC and multithreading libraries, specifically [tarpc](https://github.com/google/tarpc) and [tokio](https://docs.rs/tokio/latest/tokio/).

## Implementation Details

Reference `architecture.md`.

## References

While I have programmed in Rust, I have not build a medium-sized system with Rust. These references helped immensely in learning about the language and building a working project!
- [Plugins in Rust](https://adventures.michaelfbryan.com/posts/plugins-in-rust/)
- [Maintaining Code in Rust](https://matklad.github.io/2021/09/05/Rust100k.html)
- [Proxy for Coordinator](https://stackoverflow.com/questions/76097231/tarpc-how-to-call-outer-structs-method)

