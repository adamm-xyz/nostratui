# nostratui

**A terminal user interface (TUI) for browsing Nostr posts, written in Rust.**

nostratui is a minimalist, responsive, and highly efficient terminal client for the [Nostr](https://nostr.com) protocol.

## Installation

### Clone and Build

```bash
git clone https://github.com/adamm-xyz/nostratui.git
cd nostratui
cargo build
```

### Config

You must create a config.json at:
```bash
~/.config/nostratui/config.json
```
Then, you can add your private key, the list of relays you use and public keys you follow.

```json5
{
    "key":"nsec1...",
    "relays":[
        "wss://myrelay.xyz",
        "wss://nostr.example.net"
    ],
    "contacts:[
        [
            "npub1...",
            "matthew",
        ],
        [
            "npub1...",
            "mark",
        ],
        [
            "npub1...",
            "luke",
        ],
    ]
}

```

## Usage

### Run
```bash
cargo run
```

### Keybindings

| keybind | Description |
| ------- | ----------- |
| k | navigate up|
| j | navigate down|
| q | quit |

