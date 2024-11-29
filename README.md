# Unreal Pak Mod Manager

This tool is used to create a single .pak file from a collection of mods for an Unreal Engine game. The initial focus will be on supporting STALKER 2, but more games may be supported in the future.

## Purpose

Due to the way unreal games load mods, it's impossible to only take some parts of one mod and combine them with parts of another mod unless you manually unpack the .pak files and repack them. This tool automates this process by automatically unpacking the .pak files, resolving any conflicts between the mods, and repacking them into a single .pak file.

### Features

- Automatically resolves conflicts between STALKER 2 `.cfg` files on a per-value basis
- Automatically resolves conflicts between `.json` files on a per-value basis
- Automatically resolves conflicts between Unreal Engine `.ini` files on a per-value basis
- Attempts to automatically resolve conflicts for all other file types

### Example

Let [example/mods/abc/abc.json](example/mods/abc/abc.json) be a game file that we are modding.

It initially contains the following data:

```json
{ "a": 1, "b": 2, "c": 3 }
```

Then we add two mods that each make seperate changes...

[example/mods/increment_b/abc.json](example/mods/increment_b/abc.json) increments `b` by 1:

```json
{ "a": 1, "b": 3, "c": 3 }
```

[example/mods/inrement_c/abc.json](example/mods/increment_c/abc.json) increments `c` by 1:

```json
{ "a": 1, "b": 2, "c": 4 }
```

If we were to simply load these mods in order, we'd only end up with the changes from the last mod, in this case `increment_c`.

However, if we use this tool to create a modpack, we'd end up with both changes as shown in [example/staging/abc.json](example/staging/abc.json):

```json
{ "a": 1, "b": 3, "c": 4 }
```

#### Planned Features

These are planned features in rough order of priority

- Basic GUI

### Usage

The functionality of this tool is entirely driven by a single toml configuration file that is passed as a command line argument.

See [example/config.toml](example/config.toml) for an example:

Running `cargo run example/config.toml` will then create the modpack with all the mods in [example/mods](example/mods) and pack them in [example/example_modpack.pak](example/example_modpack.pak).
