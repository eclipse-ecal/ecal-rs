# Changelog

## Unreleased

## v0.2.0

### Features

- Implement `Publisher::set_id()` and `Publisher::shm_set_buffer_count()` methods (#8).

- Implement the `Publisher::send_with_time()` method (#9). This method let the caller set the time of the message, like the original `Send()` of eCAL.

- Implement the `Subscriber::on_recv_full()` method, which exposes the whole data and the buffer de-serialized as well instead of just an `Instant` (#10).

- Add support for capnproto via a feature flag (#11).

- Get optional protobuf, msgpack features compiling again (#13).

### Misc
- Updates to CI, including building and testing with all features.
