# proton-usage
Lists Steam applications that have specified a Steam Play compatibility tool.
Useful for when you want to remove/uninstall unused compatibility tools
but aren't sure which ones are in use.

## Installation

### Manual
Precompiled binaries are available from the GitHub [releases] archive.

### Arch Linux
If you're an Arch Linux (or a derivative like Manjaro) user, then you can install proton-usage from the [AUR]:
```
$ yay -S proton-usage
```

You may also get the precompiled version from the [AUR][aur-bin]:
```
$ yay -S proton-usage-bin
```

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

[releases]:https://github.com/rfvgyhn/proton-usage/releases
[AUR]: https://aur.archlinux.org/packages/proton-usage/
[aur-bin]: https://aur.archlinux.org/packages/proton-usage-bin/