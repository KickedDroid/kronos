# Usage


Build
```
bash build.sh 
```
---


Usage

```
USAGE:
    kronos [OPTIONS] [ARGS]

ARGS:
    <OUTPUT>    Output directory[default: ./]
    <NAME>      Session name[default: htb]

OPTIONS:
    -d, --disable-auto
    -h, --help            Print help information
    -V, --version         Print version information
```

Example

Run with defaults.

```
./kronos
```


Specify output path and name of machine.

```
./kronos /path/to/outputdir example
```
Will create a file or open at `/path/to/outputdir/example_kron_history.md`


Point the output dir to your Obsian Vault and disable auto flag submission.

```
export obsidian="path/to/vault"
./kronos $obsidian example --disable-auto
```


With cargo 
```
cargo run
```


With auto submission turned off
```
cargo run -- --disable-auto
```


#### config.toml

```
[htb]
api_token = "REPLACE_WITH_YOUR_API_TOKEN"
```
