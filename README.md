# pacman-json
Dump pacman packages information in JSON.

This package focuses on the local pacman database, yet it borrows some key
ingredients from https://github.com/jelly/pacquery which focuses more on the
sync databases.

## usage

By default, dump explicitly installed packages info:

```bash
pacman-json > pacman-explicits.json
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

Additional options can be found with `pacman-json --help`. Shell completions
generated from [**./src/completions.rs**](./src/completions.rs) are provided
under [**./completions/**](./completions/).
