FMM(5)

# NAME

fmm - configuration file

# DESCRIPTION

The config file is located at *~/.config/fmm/fmm.ini* or at
*$XDG_CONFIG_HOME/fmm/fmm.ini*. Option lines can be specified to configure fmm
like so:

```
[section_name]
key=value
```

Empty lines and lines that begin with # are ignored.

# GLOBAL OPTIONS

*mods_dir* = _string_
	The default mods directory to use. If the current directory is a valid mods
directory, it is used instead.

# SECTION: portal

To retrieve or reset your API key, token, or username, visit
https://factorio.com/profile.

*api_key* = _string_
	Your Factorio mod portal API key, used for the *upload* operation. The API key
	must have authorization for the _upload mods_ action.

*token* = _string_
	Your Factorio mod portal token, used for the *download* operation.

*username* = _string_
	Your Factorio username.

# EXAMPLE CONFIGURATION

```
mods_dir=~/.factorio/mods

[portal]
api_key=12345
token=12345
username=raiguard
```
