# meshcorrode

Meshcorrode is a library for interacting with [meshcore](https://meshcore.co.uk/) companion 
devices over BLE, serial (planned) or wifi (planned).

Meshcorrode can help you build an application that runs on top of Meshcore or otherwise interact
with your radio without using the official apps. See the [examples](examples/) directory 
for scripts that demonstrate usage.

Application -> meshcorrode -> BLE/Serial/Wifi -> Companion Device -> Meshcore network.

## Overview

1. [transport](src/transport/) defines the lowest layer of communication, shuttling raw bytes to and from the
companion device using bluetooth LE, serial or WiFi.
2. [commands](src/commands) define commands which you can send to your device. For example: "Send message to 
DE:AF:BE:EF:12:34".
3. [proto](src/proto) provides definitions of the data the companion sends back, and a parser to covert the into Event structs.
4. [event_bus.rs](src/event_bus.rs) orchestrates sending commands and receiving events, including asynchronous ones.
5. [connection](src/connection.rs) sets up the transport layer and an event bus, and provides high level methods
for sending commands and subscribing to event streams. You will do most of your interaction with meshcorrode here.
