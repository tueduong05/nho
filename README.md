# nho

A minimalist, high-performance Redis-compatible server built in Rust.

## 🚀 Features

- **Asynchronous Architecture**: Handles thousands of concurrent connections efficiently.
- **Concurrent Key-Value Store**: Thread-safe data management with high-throughput access.
- **RESP Protocol Support**: Native implementation of the Redis Serialization Protocol.
- **Memory Efficient**: Zero-copy byte management and optimized buffer handling.
- **Key Expiration**: Built-in TTL support for automatic data eviction.
- **Automatic Cleanup Worker**: Background task periodically removes expired keys to keep memory usage low.

## 🛠 Supported Commands

| Command  | Syntax                            | Description                                            |
| :------- | :-------------------------------- | :----------------------------------------------------- |
| **PING** | `PING`                            | Check server health; returns `OK`.                     |
| **SET**  | `SET key value [EX sec \| PX ms]` | Store a string value with optional expiration.         |
| **GET**  | `GET key`                         | Retrieve a value; returns `Nil` if expired or missing. |

## 🚦 Getting Started

```bash
git clone https://github.com/tueduong05/nho.git
cd nho
cargo run
```
