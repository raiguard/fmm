FMM(1)

# NAME

fmm - a mod manager for factorio

# SYNOPSIS

*fmm* <operation> [options]

# OPERATIONS

See *fmm*(5) for an explanation of the [mods] format.

*clean, c*
	Remove out-of-date mod versions, leaving only the newest versions. Ignores
	symlinked mods.

*disable, d* [mods]
	Disable the specified mods, or disable all mods if none are specified.

*download, dl* [mods]
	Download the specified mods and their dependencies.

*enable, e* [mods]
	Enable the specified mods and their dependencies.

*enable-set, es* <set>
	Enable the specified mod set - see *fmm(5)*.

*query, q* [mods]
	Query the local mods directory and display all matching mod versions.

*remove, r* [mods]
	Remove the specified mods from the local mods directory.

*search, s* <query>
	Search the mod portal for the specified query. Mod name, author, and
	description will be searched.

*sync, s* [mods]
	Enable the specified mods and their dependencies, downloading if necessary,
	and disable all other mods.

*sync-file, sf* <path>
	Enable the mods from the specified save file, downloading if necessary, disable
	all other mods, and sync mod startup settings.

*sync-list, sl* <path>
	Enable the mods from the specified *mod-list.json*, downloading if necessary,
	and disable all other mods

*sync-set, ss* <set>
	Enable the mods from the specified mod set, downloading if necessary, and
	disable all other mods. See *fmm(5)*.

*update, u* [mods]
	Update the specified mods to the latest versions, or update all mods if none
	are specified.

*upload, ul* <path>
	Upload the specified mod zip file to the mod portal.

# OPTIONS

*--help*
	Show help message and quit.

*--config* <config>
	Path to a custom configuration file.

*--game-dir* <path>
	Path to a custom game directory.

*--mods-dir* <path>
	Path to a custom mods directory.

*--token <token>*
	OAUTH token for the mod portal.

## CLEAN OPTIONS

*--all*
	Remove all non-symlinked mods instead of keeping the latest versions.

## DOWNLOAD OPTIONS

*--force, -f*
	Always download even when the mod is already present. The newly downloaded mod
	will overwrite the existing one. Will not overwrite folder or symlinked mods.

## SYNC OPTIONS

*--nodisable, -d*
	Keep current mods enabled instead of disabling them.

# ENVIRONMENT

*FMM_TOKEN*
	OAUTH token for the mod portal. Overrides the *token* configuration setting,
	but may be overridden by the *--token* option

# MODS DIRECTORY

fmm determines the mods directory in the following order:
	- the current working directory
	- the *--mods-dir* option
	- the *mods_dir* setting
	- inferred from the *--game-dir* option
	- inferred from the *game_dir* setting

A valid mods directory must have a _mod-list.json_ file and a _mod-settings.dat_
file.

# CONFIGURATION

On startup, fmm will look for a file at *$XDG_CONFIG_HOME/fmm/fmm.toml*. If
unset, *$XDG_CONFIG_HOME* defaults to *~/.config/*. Passing the *--config*
option will override this configuration file with the one specified.

See *fmm*(5) for the configuration file syntax.

# AUTHORS

Maintained by Caleb Heuer <ch@raiguard.me>. For more information about fmm
development, see https://sr.ht/~raiguard/fmm.
