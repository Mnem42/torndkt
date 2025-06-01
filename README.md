# torndkt

A desktop tool for torn, mainly designed for ranked warring

## Install instructions

### Windows

1. Select a release. Using the most recent one is reccomended.
2. Download the .exe file
3. Ideally, put the .exe file in its own folder. This is so that the persistence
   file doesn't end up getting mixed up.
4. Make desktop shortcut to .exe (optional)

### Build from source

```bash
git clone https://github.com/Mnem42/torndkt
cd torndkt
cargo build torndkt
```

Optionally, the `--release` flag can be passed to cargo so that it builds as
release instead of debug.