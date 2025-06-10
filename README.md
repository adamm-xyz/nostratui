# nostratui

**A terminal user interface (TUI) for browsing Nostr posts, written in Rust.**


<div align="center">
  <p>
    <a href="https://github.com/adamm-xyz/nostratui" target="_blank">
      <img width="100%" src="https://raw.githubusercontent.com/adamm-xyz/nostratui/refs/heads/main/demo.gif" alt="nostratui demo banner"></a>
  </p>
</div>
nostratui is a minimalist, responsive, and highly efficient terminal client for the <a href="https://nostr.com">nostr</a> protocol.

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
    "contacts":[
        [
            "npub1...",
            "matthew"
        ],
        [
            "npub1...",
            "mark"
        ],
        [
            "npub1...",
            "luke"
        ]
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
| Ctrl-r | refresh feed|
| n | create new post|
| r | reply to selected post|
| q | quit |

## Roadmap
- [x] NIP-01, fetch and display basic notes
- [x] NIP-02, fetch follow list
- [ ] Display contacts page
- [ ] NIP-05 display user name handles
- [ ] Key generation with NIP-06
- [ ] NIP-08 display mentions in posts
- [ ] NIP-09 issue delete requests
- [x] NIP-10 show note threads (still WIP)
