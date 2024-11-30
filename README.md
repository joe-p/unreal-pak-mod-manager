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

1. Download the latest release from the latest GitHub release: https://github.com/joe-p/unreal-pak-mod-manager/releases
2. Place the `unreal-pak-mod-manager.exe` in any folder you'd like. This is where all the mods and final modpack will be saved. For example: `C:\MyModpack`
3. Double click the `unreal-pak-mod-manager.exe` to run it (or run it from the terminal) and you should see

```
Created default config file: \\?\C:\MyModpack\config.toml
Created mods directory, put pak files here and run this program again to create a modpack: \\?\C:\MyModpack\mods
Press Enter to exit..
```

4. Place all of your `.pak` files you wish to add to your modpack in the `mods` folder next to the `unreal-pak-mod-manager.exe`
5. Run `unreal-pak-mod-manager.exe` again and you should see:

```
Processing the mods in the following order:
0: mods\ZZFrancisLouisSOVer2_P.pak
1: mods\zzz_Grok_Bloodsucker-60percent_HP_P.pak
2: mods\zzz_Grok_Boar-40percent_HP_P.pak
3: mods\zzz_Grok_Burer-40percent_HP_P.pak
4: mods\zzz_Grok_Cat-40percent_HP_P.pak
5: mods\zzz_Grok_Chimera-60percent_HP_P.pak
6: mods\zzz_Grok_Controller-40percent_HP_P.pak
7: mods\zzz_Grok_Deer-20percent_HP_P.pak
8: mods\zzz_Grok_Flesh-40percent_HP_P.pak
9: mods\zzz_Grok_Poltergeist-60percent_HP_P.pak
10: mods\zzz_Grok_PseudoDog-60percent_HP_P.pak
11: mods\zzz_Grok_Pseudogiant-40percent_HP_P.pak
12: mods\~S2optimizedTweaksBASE_v1.91_P.pak
ZZFrancisLouisSOVer2_P.pak: Extracting Engine/Config/Windows/WindowsEngine.ini
ZZFrancisLouisSOVer2_P.pak: Extracting MADE_BY_FRANCISLOUIS.txt
zzz_Grok_Bloodsucker-60percent_HP_P.pak: Extracting Stalker2/Content/GameLite/GameData/ObjPrototypes/Bloodsucker.cfg
zzz_Grok_Boar-40percent_HP_P.pak: Extracting Stalker2/Content/GameLite/GameData/ObjPrototypes/Boar.cfg
zzz_Grok_Burer-40percent_HP_P.pak: Extracting Stalker2/Content/GameLite/GameData/ObjPrototypes/Burer.cfg
zzz_Grok_Cat-40percent_HP_P.pak: Extracting Stalker2/Content/GameLite/GameData/ObjPrototypes/Cat.cfg
zzz_Grok_Chimera-60percent_HP_P.pak: Extracting Stalker2/Content/GameLite/GameData/ObjPrototypes/Chimera.cfg
zzz_Grok_Controller-40percent_HP_P.pak: Extracting Stalker2/Content/GameLite/GameData/ObjPrototypes/Controller.cfg
zzz_Grok_Deer-20percent_HP_P.pak: Extracting Stalker2/Content/GameLite/GameData/ObjPrototypes/Deer.cfg
zzz_Grok_Flesh-40percent_HP_P.pak: Extracting Stalker2/Content/GameLite/GameData/ObjPrototypes/Flesh.cfg
zzz_Grok_Poltergeist-60percent_HP_P.pak: Extracting Stalker2/Content/GameLite/GameData/ObjPrototypes/Poltergeist.cfg
zzz_Grok_PseudoDog-60percent_HP_P.pak: Extracting Stalker2/Content/GameLite/GameData/ObjPrototypes/PseudoDog.cfg
zzz_Grok_Pseudogiant-40percent_HP_P.pak: Extracting Stalker2/Content/GameLite/GameData/ObjPrototypes/Pseudogiant.cfg
~S2optimizedTweaksBASE_v1.91_P.pak: Extracting Engine/Config/Windows/WindowsEngine.ini
~S2optimizedTweaksBASE_v1.91_P.pak: Extracting Stalker2/A message.jpg
ZZFrancisLouisSOVer2_P_pak: Merging with priority 0
ZZFrancisLouisSOVer2_P_pak: Merging files
ZZFrancisLouisSOVer2_P_pak: All files merged without conflicts
zzz_Grok_Bloodsucker-60percent_HP_P_pak: Merging with priority 1
zzz_Grok_Bloodsucker-60percent_HP_P_pak: Merging files
zzz_Grok_Bloodsucker-60percent_HP_P_pak: All files merged without conflicts
zzz_Grok_Boar-40percent_HP_P_pak: Merging with priority 2
zzz_Grok_Boar-40percent_HP_P_pak: Merging files
zzz_Grok_Boar-40percent_HP_P_pak: All files merged without conflicts
zzz_Grok_Burer-40percent_HP_P_pak: Merging with priority 3
zzz_Grok_Burer-40percent_HP_P_pak: Merging files
zzz_Grok_Burer-40percent_HP_P_pak: All files merged without conflicts
zzz_Grok_Cat-40percent_HP_P_pak: Merging with priority 4
zzz_Grok_Cat-40percent_HP_P_pak: Merging files
zzz_Grok_Cat-40percent_HP_P_pak: All files merged without conflicts
zzz_Grok_Chimera-60percent_HP_P_pak: Merging with priority 5
zzz_Grok_Chimera-60percent_HP_P_pak: Merging files
zzz_Grok_Chimera-60percent_HP_P_pak: All files merged without conflicts
zzz_Grok_Controller-40percent_HP_P_pak: Merging with priority 6
zzz_Grok_Controller-40percent_HP_P_pak: Merging files
zzz_Grok_Controller-40percent_HP_P_pak: All files merged without conflicts
zzz_Grok_Deer-20percent_HP_P_pak: Merging with priority 7
zzz_Grok_Deer-20percent_HP_P_pak: Merging files
zzz_Grok_Deer-20percent_HP_P_pak: All files merged without conflicts
zzz_Grok_Flesh-40percent_HP_P_pak: Merging with priority 8
zzz_Grok_Flesh-40percent_HP_P_pak: Merging files
zzz_Grok_Flesh-40percent_HP_P_pak: All files merged without conflicts
zzz_Grok_Poltergeist-60percent_HP_P_pak: Merging with priority 9
zzz_Grok_Poltergeist-60percent_HP_P_pak: Merging files
zzz_Grok_Poltergeist-60percent_HP_P_pak: All files merged without conflicts
zzz_Grok_PseudoDog-60percent_HP_P_pak: Merging with priority 10
zzz_Grok_PseudoDog-60percent_HP_P_pak: Merging files
zzz_Grok_PseudoDog-60percent_HP_P_pak: All files merged without conflicts
zzz_Grok_Pseudogiant-40percent_HP_P_pak: Merging with priority 11
zzz_Grok_Pseudogiant-40percent_HP_P_pak: Merging files
zzz_Grok_Pseudogiant-40percent_HP_P_pak: All files merged without conflicts
_S2optimizedTweaksBASE_v1_91_P_pak: Merging with priority 12
_S2optimizedTweaksBASE_v1_91_P_pak: Merging files
_S2optimizedTweaksBASE_v1_91_P_pak: All files merged without conflicts
upmm_modpack.pak: Packing Engine/Config/Windows/WindowsEngine.ini
upmm_modpack.pak: Packing MADE_BY_FRANCISLOUIS.txt
upmm_modpack.pak: Packing Stalker2/A message.jpg
upmm_modpack.pak: Packing Stalker2/Content/GameLite/GameData/ObjPrototypes/Bloodsucker.cfg
upmm_modpack.pak: Packing Stalker2/Content/GameLite/GameData/ObjPrototypes/Boar.cfg
upmm_modpack.pak: Packing Stalker2/Content/GameLite/GameData/ObjPrototypes/Burer.cfg
upmm_modpack.pak: Packing Stalker2/Content/GameLite/GameData/ObjPrototypes/Cat.cfg
upmm_modpack.pak: Packing Stalker2/Content/GameLite/GameData/ObjPrototypes/Chimera.cfg
upmm_modpack.pak: Packing Stalker2/Content/GameLite/GameData/ObjPrototypes/Controller.cfg
upmm_modpack.pak: Packing Stalker2/Content/GameLite/GameData/ObjPrototypes/Deer.cfg
upmm_modpack.pak: Packing Stalker2/Content/GameLite/GameData/ObjPrototypes/Flesh.cfg
upmm_modpack.pak: Packing Stalker2/Content/GameLite/GameData/ObjPrototypes/Poltergeist.cfg
upmm_modpack.pak: Packing Stalker2/Content/GameLite/GameData/ObjPrototypes/PseudoDog.cfg
upmm_modpack.pak: Packing Stalker2/Content/GameLite/GameData/ObjPrototypes/Pseudogiant.cfg
upmm_modpack.pak created successfully!
Press Enter to exit...
```

6. Move `upmm_modpack.pak` to the `~mods` folder. The default location for Steam installs is `C:\Program Files (x86)\Steam\steamapps\common\S.T.A.L.K.E.R. 2 Heart of Chornobyl\Stalker2\Content\Paks\~mods`

**Note:** You can also use the `copy_to_dir` in the `config.toml` to automate the last step of copying it into `~mods`

### Configuration

To control the order of mods, the directories used by the program, and the name of the final modpack you can modify the `config.toml`. See [example/config.toml](example/) for an example configuration file that explains all the options (note the mods here are nonsensical and are only for example purposes).

## FAQs

### Why use this tool?

The main benefit this tool has over other tools is that it resolves conflicts between mods on a per-value basis rather than regular (or manual) merge conflict resolution. This makes the merge process more robust and able to handle more complex changes. This also means that non-functional changes, such as changing comments or moving lines, will not affect the outcome of the merge. It also does not have any external dependencies since it is written in Rust and able to use the [repak](https://github.com/trumank/repak) library directly.

### Why no GUI?

I personally don't have much GUI experience and do not want to sink time into creating one where there is still a lot of work to be done on the core functionality. Because this mod tool is a single binary that is driven by a single TOML file anyone is more than welcome to create their own GUI for it using their language of choice or contribute a GUI to this project. The GUI would simply need to read/write the config file and then spawn the tool as a subprocess. Once I am happy with the core functionality, I will begin to create a GUI for it if one has not already been created.

### Why am I getting an error about struct begin/end?

Some cfg files that are shipped with the game are *seemingly* malformed. The most common thing is a missing `stuct.end` like in `Stalker2/Content/GameLite/GameData/Scripts/OnGameLaunch/OnGameLaunchScripts_EQ155.cfg`:


```
[0] : struct.begin
   SID = OnGameLaunchScripts_EQ155
   ScriptsArray : struct.begin
      [*] = XSetGlobalBool DBG_IsDebug 1
      [*] = XStartQuestNodeBySID EQ155_P_Technical_DebugStart
   struct.end
```

It's entirely possible that this is the intended syntax of CFG files, but since it's so rare I am assuming it is not intentional thus the parser will throw an error when encoutering these files. Since I doubt these are commonly modded files, that seems to be acceptable for now. I have ran this tool against every single file in `pakchunk0` to ensure every file can be parsed properly. Below are the ones that will cause the tool to throw an error:


```
./Stalker2/Content/GameLite/GameData/ArtifactPrototypes/QuestArtifactPrototypes.cfg
./Stalker2/Content/GameLite/GameData/ItemGeneratorPrototypes/GDItemGeneratorPrototype/VortexDudeItemGenerator.cfg
./Stalker2/Content/GameLite/GameData/Scripts/OnGameLaunch/OnGameLaunchScripts_E01_MQ01_NoIntro.cfg
./Stalker2/Content/GameLite/GameData/Scripts/OnGameLaunch/OnGameLaunchScripts_E16_Bossfight_Scar.cfg
./Stalker2/Content/GameLite/GameData/Scripts/OnGameLaunch/OnGameLaunchScripts_EQ152_Spark.cfg
./Stalker2/Content/GameLite/GameData/Scripts/OnGameLaunch/OnGameLaunchScripts_EQ152_Ward.cfg
./Stalker2/Content/GameLite/GameData/Scripts/OnGameLaunch/OnGameLaunchScripts_EQ154.cfg
./Stalker2/Content/GameLite/GameData/Scripts/OnGameLaunch/OnGameLaunchScripts_EQ155.cfg
```

If you are using a mod that contains one of these files or encounter similar syntax in the wild please let me know!

### Why does Windows defender warn me when I try to use this tool?

By default, Windows defender will warn you any time you use an unsigned exe for the first time. This DOES NOT mean it is a virus, it's just Windows making sure you know what you are running and should go away after the first time you run it. I could fix this by signing the exe, but I need to pay for a proper signing key. If this tool sees adoption, I will look into signing it to avoid this. If you are still skeptical, you can clone this repo yourself and build via `cargo build --release --target x86_64-pc-windows-gnu`


### How can I see what changes were made?

By default, the tool will create a `staging` directory that contains all of the files before they are packed. You can look at these files to see the final result that is in the modpack. This `staging` directory is a git repository, so you can also use `git` to view a history of how the files changed over time as mods were merged in. I eventually plan to add an easier way to review changes for those that aren't familiar with git.