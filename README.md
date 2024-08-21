# nu_plugin_bash_env

A Bash environment plugin for Nushell.

For instructions on how to use this plugin, see the [Nushell book](https://www.nushell.sh/book/plugins.html).

In summary, save the `nu_plugin_bash_env` script in your Nu plugins directory (for example) and ensure it is executable.
For users of Nix, this is now installable as a flake (see below).

Then in Nu:
```
> register nu_plugin_bash_env
```

The plugin reads the specified environment file (if any) and evaluates variables from `stdin` (if any) and returns any new or changed environment variables as a record, suitable for passing to Nu's `load-env`.

## Plugin Version Compatability

Since Nushell 0.91.0 the plugin protocol was enhanced and now requires version compatability between plugins and Nushell itself.

The following versions are compatible.

| Nushell | bash-env plugin |
| ------- | --------------- |
|    0.89 |           0.5.0 |
|    0.90 |           0.5.0 |
|    0.91 |           0.6.2 |
|    0.92 |           0.7.1 |
|    0.93 |           0.8.0 |
|    0.93 |           0.9.0 |
|    0.94 |          0.10.0 |
|    0.95 |          0.11.0 |
|    0.96 |          0.12.1 |
|    0.97 |          0.13.0 |

If you find a new version of Nushell rejects this plugin as incompatible, please report an [issue](https://github.com/tesujimath/nu_plugin_bash_env/issues).

## Dependencies

The script uses `jq` for heavy lifting.  [At least jq version 1.7 is required](https://github.com/tesujimath/nu_plugin_bash_env/issues/24).

Also I suspect at least Bash version 5.1.

## Examples

### Simple Usage
```
> bash-env tests/simple.env
╭───┬───╮
│ B │ b │
│ A │ a │
╰───┴───╯

> echo $env.A
Error: nu::shell::name_not_found

  × Name not found


> bash-env tests/simple.env | load-env

> echo $env.A
a
> echo $env.B
b


> bash-env tests/simple.env
╭──────────────╮
│ empty record │
╰──────────────╯

# no new or changed environment variables, so nothing returned

> ssh-agent | bash-env
Agent pid 98985
╭───────────────┬───────────────────────────────────╮
│ SSH_AUTH_SOCK │ /tmp/ssh-XXXXXXFIMT9y/agent.98982 │
│ SSH_AGENT_PID │ 98985                             │
╰───────────────┴───────────────────────────────────╯
```

### Exporting Shell Variables

The plugin supports `--export` for exporting shell variables into the environment.

```
> echo "ABC=123" | bash-env
╭──────────────╮
│ empty record │
╰──────────────╯

> echo "export ABC=123" | bash-env
╭─────┬─────╮
│ ABC │ 123 │
╰─────┴─────╯

> echo "ABC=123" | bash-env --export [ABC]
╭─────┬─────╮
│ ABC │ 123 │
╰─────┴─────╯

> bash-env /etc/os-release
╭──────────────╮
│ empty record │
╰──────────────╯

> bash-env --export [ID PRETTY_NAME] /etc/os-release
╭─────────────┬──────────────────────╮
│ ID          │ nixos                │
│ PRETTY_NAME │ NixOS 24.05 (Uakari) │
╰─────────────┴──────────────────────╯
```

### Escaping Special Characters

Care has been taken to escape any special characters.

```
> bash-env `tests/Ming's "menu" of (merciless) monstrosities.env`
╭───────────┬──────────────────────────────────────────────────────╮
│ QUOTE     │ "Well done!" is better than "Well said!"             │
│ SPACEMAN  │ One small step for a man ...                         │
│ MIXED_BAG │ Did the sixth sheik's sixth sheep say "baa", or not? │
╰───────────┴──────────────────────────────────────────────────────╯

> bash-env `tests/Ming's "menu" of (merciless) monstrosities.env` | load-env
> echo $env.QUOTE
"Well done!" is better than "Well said!"
```

## API

Because this plugin is written in Bash, the API signatures must be written by hand.  The [api](api) sub-directory contains a Rust program to produce what is required, using the official Nu plugin library.

## Nix flake

The plugin is installable from its flake using Nix Home Manager.

See my own [Home Manager flake](https://github.com/tesujimath/home.nix/blob/main/flake.nix#L12) and [nushell module](https://github.com/tesujimath/home.nix/blob/main/modules/nushell/default.nix) for hints how to achieve this.  Note in particular the requirement for [each-time plugin registration](https://github.com/tesujimath/home.nix/blob/main/modules/nushell/config.nu#L761).

## Notes

All local variables in the script are prefixed with underscore, in an attempt to mitigate Bash's inability to distinguish variables local to the shell and environment variables, but this is by no means bulletproof.

## Future work

- unsetting an environment variable ought to be possible
