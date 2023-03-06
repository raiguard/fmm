FMM(1)

# NAME

fmm - a mod manager for factorio

# SYNOPSIS

*fmm* [options] <operation> args...

# MOD SYNTAX

Operations and settings that take _mod_ values expect a string in the format of
*name* or *name_version*. If _version_ is omitted, it is assumed to be the
newest version in the local mods directory (for local operations) or the newest
version on the mod portal (for remote operations).

_version_ takes the format of *major.minor.patch*, e.g. *1.1.3*. Leading zeroes
will be ignored during comparison.

## EXAMPLES

```
RecipeBook
RecipeBook_3.5.2
RecipeBook_4.0
```

# OPERATIONS

*disable, d* [mods...]
	Disable the specified mods, or disable all mods if none are specified.

*download, dl* [mods...]
	Download the specified mods and their dependencies.

*enable, e* [mods...]
	Enable the specified mods and their dependencies.

*upload, ul* [files...]
	Upload the specified mod zip files to the mod portal.

# OPTIONS

*-c* <file>
	Path to a custom configuration file.

*-h*
	Show help message and quit.

# ENVIRONMENT

Environment variables will be prioritized over options set in the config file,
but can be overridden by CLI options.

To retrieve or reset your API key, token, or username, visit
https://factorio.com/profile.

*FACTORIO_API_KEY*
	Your Factorio mod portal API key, used for the *upload* operation. The API key
	must have authorization for the _upload mods_ action.

*FACTORIO_TOKEN*
	Your Factorio mod portal token, used for the *download* operation.

*FACTORIO_USERNAME*
	Your Factorio username.

# MODS DIRECTORY

fmm determines the mods directory in the following order:
	- the current working directory
	- the *mods_dir* setting

A valid mods directory must have a _mod-list.json_ file.

# CONFIGURATION

See *fmm*(5) for the configuration file syntax.

# AUTHORS

Maintained by Caleb Heuer <ch@raiguard.me>. For more information about fmm
development, see https://github.com/raiguard/fmm.
