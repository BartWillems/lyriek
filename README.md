# Lyriek

A multi-threaded GTK 3 application for fetching the lyrics of the current playing song.

![lyriek window](screenshots/lyriek-window.png)

## Installation

### Arch Linux

```
yay -S lyriek
```

### Ubuntu

```
apt install libgtk-3-dev --no-install-recommends
cargo build --release
```

## Troubleshooting

### No Active Player

Lyriek uses [MPRIS](https://wiki.archlinux.org/index.php/MPRIS) to get song information.

If you see `no active player`, that means that you either have no media player running, or your media player doesn't communicate to D-Bus using MPRIS.
