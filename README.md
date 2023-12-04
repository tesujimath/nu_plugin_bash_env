# nu_plugin_bash_env

A Bash environment plugin for Nushell.

For instructions on how to use this plugin, see the [Nushell book](https://www.nushell.sh/book/plugins.html).

In summary, save the `nu_plugin_bash_env` script somewhere on your path and ensure it is executable.

Then in Nu:
```
> register nu_plugin_bash_env
```

The plugin reads the specified environment file and returns any new or changed environment variables as a record, suitable for passing to Nu's `load-env`.

## Dependencies

The script uses `jq` for heavy lifting.

## Examples

### Simple Usage
```
> bash-env examples/simple.env
╭───┬───╮
│ B │ b │
│ A │ a │
╰───┴───╯

> echo $env.A
Error: nu::shell::name_not_found

  × Name not found


> bash-env examples/simple.env | load-env

> echo $env.A
a
> echo $env.B
b


> bash-env examples/simple.env
╭──────────────╮
│ empty record │
╰──────────────╯

# no new or changed environment variables, so nothing returned

```

### Escaping Special Characters

Care has been taken to escape any special characters.

```
> bash-env `examples/Ming's "menu" of (merciless) monstrosities.env`
╭───────────┬──────────────────────────────────────────────────────╮
│ QUOTE     │ "Well done!" is better than "Well said!"             │
│ SPACEMAN  │ One small step for a man ...                         │
│ MIXED_BAG │ Did the sixth sheik's sixth sheep say "baa", or not? │
╰───────────┴──────────────────────────────────────────────────────╯

> bash-env `examples/Ming's "menu" of (merciless) monstrosities.env` | load-env
> echo $env.QUOTE
"Well done!" is better than "Well said!"
```

## Notes

All local variables in the script are prefixed with underscore, in an attempt to mitigate Bash's inability to distinguish variables local to the shell and environment variables, but this is by no means bulletproof.


## Future work

- unsetting an environment variable ought to be possible
- package as an installable Nix flake
