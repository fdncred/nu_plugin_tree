# nu_plugin_tree

This is a [Nushell](https://nushell.sh/) plugin called "tree". It takes nushell values and transforms them into a tree shape.

## Installing

```nushell
> cargo install --path .
```

## Usage

```nushell
> plugin add ~/.cargo/bin/nu_plugin_tree
> plugin use tree
> tree --help
View the contents of the pipeline as a tree.

Usage:
  > tree

Flags:
  -h, --help: Display the help message for this command

Examples:
  Transform the tabular output into a tree
  > scope commands | where name == with-env | tree

  Transform the tabular output into a tree
  > ls | tree
```
