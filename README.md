![Build](http://localhost:43110/1H3ct93gHL9BgtTnyrqJrkjn4NdociFFTn/img/build.svg)
![Tests](http://localhost:43110/1H3ct93gHL9BgtTnyrqJrkjn4NdociFFTn/img/tests.svg)
![Coverage](http://localhost:43110/1H3ct93gHL9BgtTnyrqJrkjn4NdociFFTn/img/coverage.svg)

# ZeroNet Protocol Rust Library
[Documentation](http://localhost:43110/1H3ct93gHL9BgtTnyrqJrkjn4NdociFFTn/doc/zeronet_protocol)


# Roadmap
- [x] Implementation covers basic use-cases
- [x] Make send and receive async
- [ ] Parse addresses
	- [x] IPV4, IPV6 & OnionV2
	- [ ] OnionV3, I2P, LokiNet
- [ ] Pack and unpack addresses
	- [x] IPV4, IPV6 & OnionV2
	- [ ] OnionV3, I2P, LokiNet
- [ ] Support receiving of the streamFile response
- [ ] Add configurable timeouts
- [ ] Add serialization and deserialization tests for all message types used by ZeroNet-py3 and ZeroNet trackers.
- [ ] Provide templates for all standard ZeroNet messages.
- [ ] Write documentation that covers all outwards facing structs, traits and functions.
	- [ ] ZeroConnection
	- [ ] ZeroMessage, Response, Request
- [ ] Proper Error handling
- [ ] Optimalization
	- [ ] Fixing the rmp-serde bug resulting in UnknownLength error will allow us to encode without passing through serde_json first, this should result in a significant performance boost.
	- [ ] Benchmark the serialization and deserialization of the intermediary custom type used for the request parameters and response values, currently it uses HashMap, this is highly unlikely to be optimal.
	- [ ] Replace serde_json::Number in the custom value so that serde_json can be cut completely from the dependencies when the previous bug is fixed.
