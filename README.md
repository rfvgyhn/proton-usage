# proton-usage
Lists Steam applications that have specified a Steam Play compatibility tool.
Useful for when you want to remove/uninstall unused compatibility tools
but aren't sure which ones are in use.

## Usage
```
USAGE:
    proton-usage [OPTIONS]

OPTIONS:
    -c, --config-path <CONFIG_PATH>    Path to the config.vdf file. Default: ~/.steam/root/config/config.vdf
    -h, --help                         Print help information
    -v, --verbose                      Output verbosity (-v, -vv, -vvv, etc)
    -V, --version                      Print version information
```

## Preview
```
user@arch:~$ proton-usage
Proton-6.0-GE-1
    F1® 2020
Proton-6.10-GE-1
    Sea of Thieves
Proton-6.5-GE-2
    Divinity: Original Sin 2 - Definitive Edition
Proton-6.8-GE-2
    Thronebreaker: The Witcher Tales
    Tainted Grail: Conquest
proton_411
    We Were Here
    XCOM®: Chimera Squad
proton_5
    Supreme Commander: Forged Alliance
proton_experimental
    Ragnarock
```