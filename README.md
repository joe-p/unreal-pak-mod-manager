# Unreal Pak Mod Manager

This tool is used to create a single .pak file from a collection of mods for an Unreal Engine game. The initial focus will be on supporting STALKER 2 pak mods (notably cfg mods and .ini mods), but more games and file formats may be supported in the future.

## Purpose

Due to the way unreal games load mods, it's impossible to only take some parts of one mod and combine them with parts of another mod unless you manually unpack the .pak files and repack them. This tool automates this process by automatically unpacking the .pak files, resolving any conflicts between the mods, and repacking them into a single .pak file.

## Features

- Single binary with no dependencies
- Automatically resolves conflicts between STALKER 2 `.cfg` files on a per-value basis
- Automatically resolves conflicts between `.json` files on a per-value basis
- Automatically resolves conflicts between Unreal Engine `.ini` files on a per-value basis
- Attempts to automatically resolve conflicts for all other file types

## Usage

The functionality of this tool is entirely driven by a single toml configuration file that is passed as a command line argument. If a configuration file is not provided, the tool will create a default config in the current directory and setup the necessary directories.

For a complete example, see [example/config.toml](example/) for an example configuration and modpack (note the mods here are nonsensical and are only for example purposes):

Running `cargo run example/config.toml` from the root of this repository will create the modpack with all the mods in [example/mods](example/mods), stage them in [example/staging](example/staging), and pack them in [example/example_modpack.pak](example/example_modpack.pak).

## FAQs

### Why Use This Tool?

The main benefit this tool has over other tools is that it resolves conflicts between mods on a per-value basis rather than regular (or manual) merge conflict resolution. This makes the merge process more robust and able to handle more complex changes. This also means that non-functional changes, such as changing comments or moving lines, will not affect the outcome of the merge. It also does not have any external dependencies since it is written in Rust and able to use the [repak](https://github.com/trumank/repak) library directly.

### Why No GUI?

I personally don't have much GUI experience and do not want to sink time into creating one where there is still a lot of work to be done on the core functionality. Because this mod tool is a single binary that is driven by a single TOML file anyone is more than welcome to create their own GUI for it using their language of choice or contribute a GUI to this project. The GUI would simply need to read/write the config file and then spawn the tool as a subprocess. Once I am happy with the core functionality, I will begin to create a GUI for it if one has not already been created.
