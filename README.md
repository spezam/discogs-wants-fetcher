# discogs-wants-fetcher 🦀

Fetches and displays a user's [Discogs](https://www.discogs.com) wantlist, handling pagination and rate limiting automatically.

## Install

```sh
cargo install --path .
```

## Usage

```sh
discogs-wants-fetcher --username <discogs-username>
```

Output:

```
Artist Name — Album Title (Year)
```

## Build from source

```sh
cargo build --release
./target/release/discogs-wants-fetcher --username <username>
```
