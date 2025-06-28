# journal

A thing to focus on the writing and not the reading. The anti-Roam, the anti-Obsidian.

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

## Implementation

Rust, Sqlite, Htmx
