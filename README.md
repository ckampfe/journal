# journal

A thing to focus on the writing, not the reading. The anti-Obsidian, the anti-Roam.

## It looks like this

<img width="727" alt="Screenshot 2025-06-28 at 17 39 29" src="https://github.com/user-attachments/assets/8906e3a8-9c3c-45b9-9716-f3d29a6ea360" />

## Running

```sh
Usage: journal [OPTIONS]

Options:
      --database-path <DATABASE_PATH>  [env: JOURNAL_DB_PATH=]
      --port <PORT>                    [env: PORT=] [default: 9999]
  -h, --help                           Print help
```

## Building

`$ cargo build [--release]`

## Pro tips

Set up a keyboard shortcut to open `http://localhost:9999` or wherever you have this running.
Set up Caddy to give it a name, like:

```
# in Caddyfile
journal.localhost {
  reverse_proxy :9999
}
```

## Motivation

My hypothesis is that Obsidian, Roam, and all of the "knowledge base" software popular these days see the reading vs. writing value breakdown as being something like 90:10%. I think it's the inverse. The main behaviors Obsidian/Roam/etc. encourage are making backlinks, tinkering with their settings, and trying to come up with a cogent directory structure. Accordingly, they're great for things like lists, or documents you need to frequently reference. They're bad for thinking or journaling, because the temptation to fritter around making backlinks and tweaking things that don't matter is too great due to what their designs incentivize.

## Implementation

Rust, Sqlite, Htmx
