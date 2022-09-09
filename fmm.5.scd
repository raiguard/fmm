fmm(5)

# NAME

fmm - a mod manager for factorio

# DESCRIPTION

The fmm configuration file uses the *TOML* file format. For more details on the
TOML syntax, see https://toml.io/en/.

All settings are optional.

# GLOBAL OPTIONS

*game_dir* = _string_
	The default game directory to use.

*mods_dir* = _string_
	The default mods directory to use.

*sync_latest_versions* = _boolean_
	If enabled, the *sync-file* operation will enable the newest version of each
	mod, instead of the exact version contained in the save file.

*token* = _string_
	Your Factorio mod portal OAUTH token. This token is currently only
	used for the *upload* operation, but will eventually replace the _portal_
	configuration section once the mod portal API is updated to allow it. The
	*FMM_TOKEN* environment variable and the *--token* option will override this
	setting.

# PORTAL CREDENTIALS

fmm will attempt to extract your mod portal credentials from the
*player-data.json* file in the game directory. If the game directory has not
been specified, or if the credentials otherwise do not exist there, you may
specify them in the *[portal]* section of the configuration file.

*token* = _string_
	Your legacy mod portal token. To retrieve or reset your token, see
	https://factorio.com/profile.

*username* = _string_
	Your Factorio username.

# MOD SETS

When using the *enable-set* or *sync-set* operations, fmm will check the
*[sets]* section of the configuration file for the set that you specified.
Each mod in the set, as well as its dependencies, will be enabled.

_string_ = _array[mod]_

## MOD SYNTAX

Operations and settings that take _mod_ values expect a string in the format of
*name[@version]*. If _version_ is omitted, it is assumed to be the newest
version in the local mods directory (for local operations) or the newest version
on the mod portal (for remote operations).

_version_ takes the format of *major.minor.patch*, e.g. *1.1.3*. Leading zeroes
will be ignored during comparison.

# EXAMPLE CONFIGURATION

```
game_dir = "~/.factorio"
sync_latest_versions = true

[portal]
token = "12345"
username = "raiguard"

[sets]
sandbox = ["EditorExtensions", "BlueprintTools"]
k2se = ["Krastorio2", "space-exploration", "EditorExtensions"]
oldk2 = ["Krastorio2@1.2.0", "EditorExtensions"]
```
