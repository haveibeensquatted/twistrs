# Twistrs gRPC Example

A twistrs client implementation using gRPC and Envoy as the transport layer.

## Demo

The example provides both an example server and client implementation. First setup the server as follows:

```
cd /path/to/twistrs
cd examples/twistrs-grpc
docker-compose build
docker-compose up
```

Once up and running, you can run the client implementation that will pass the gRPC through to Envoy on http://localhost:8080.

```
cargo r --bin client
```
  
![twistrs-grpc-example](res/twistrs-grpc-example.gif)
