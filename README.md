# ZeroNet Protocol Rust Library
[Documentation](http://localhost:43110/1H3ct93gHL9BgtTnyrqJrkjn4NdociFFTn/doc/zeronet_protocol)


# Roadmap
- [x] Implementation covers basic use-cases
- [x] Make send and receive async
- [ ] Support receiving of the streamFile response
- [ ] Add serialization and deserialization tests for all message types used by ZeroNet-py3 and ZeroNet trackers.
- [ ] Write documentation that covers all outwards facing structs, traits and functions.
  - [ ] ZeroConnection
  - [ ] ZeroMessage, Response, Request
- [ ] Proper Error handling
- [ ] Optimalization
  - [ ] Fixing the rmp-serde bug resulting in UnknownLength error will allow us to encode without passing through serde_json first, this should result in a significant performance boost.
  - [ ] Benchmark the serialization and deserialization of the intermediary custom type used for the request parameters and response values, currently it uses HashMap, this is highly unlikely to be optimal.
  - [ ] Replace serde_json::Number in the custom value so that serde_json can be cut completely from the dependencies when the previous bug is fixed.
