# Demo

A simple demo in which two agents communicate with each other.

To test it, run the demo twice (in two windows) at the same time:

    # `ping` running in window 1:
    $ RUST_LOG=info cargo run --release
    [2023-04-04T09:32:48Z INFO  ecal] eCAL initiailized as 'kcal_ping'.
    [2023-04-04T09:32:49Z INFO  ecal_ping_demo] Ping 1
    [2023-04-04T09:32:49Z INFO  ecal_ping_demo] Pong 2
    [2023-04-04T09:32:50Z INFO  ecal_ping_demo] Ping 2
    [2023-04-04T09:32:50Z INFO  ecal_ping_demo] Pong 3

    # `pong` running simultaneously in window 2:
    $ RUST_LOG=info cargo run --release -- --pong
    [2023-04-04T09:32:48Z INFO  ecal] eCAL initiailized as 'kcal_ping'.
    [2023-04-04T09:32:48Z INFO  ecal_ping_demo] Ping 1
    [2023-04-04T09:32:49Z INFO  ecal_ping_demo] Ping 1
    [2023-04-04T09:32:49Z INFO  ecal_ping_demo] Ping 1
    [2023-04-04T09:32:49Z INFO  ecal_ping_demo] Pong 2
