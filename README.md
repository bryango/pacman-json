# pacjump
**pac**man **j**son d**ump**: dump pacman packages information in JSON.

This package focuses on the local pacman database, yet it borrows some key
ingredients from https://github.com/jelly/pacquery which focuses more on the
sync databases.

> Note: this package used to have the name `pacman-json`, but it was renamed
> for version 0.2.1 to avoid sounding too official.

## usage

By default, dump explicitly installed packages info:

```bash
pacjump > pacman-explicits.json
```

One can then process the resulting JSON with `jq`, e.g. get the subset of
packages from the official repo that is _not_ maintained by
`someone@archlinux.org`:

```bash
cat pacman-explicits.json | jq --raw-output '
  .[]
    | select( .repository | test("core|extra|multilib") )
    | select( .packager | contains("@archlinux.org") | not)
    | "\(.name), \(.repository), \(.packager)"
'
```

To collect all the dependencies of a single package (in this example,
`texstudio`), and compute the size of this closure:

```bash
pacjump --recurse=texstudio \
  | jq '[ .[].installed_size ] | add' \
  | numfmt --to=iec
```

Additional options can be found with `pacjump --help`. Shell completions
generated from [**./src/completions.rs**](./src/completions.rs) are provided
under [**./completions/**](./completions/).
