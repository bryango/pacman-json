# pacman-json
dumps json data of the explicitly installed pacman packages

## usage

```bash
pacman-json > pacman-explicits.json

## get the subset of the official packages
## ... that is not maintained by `someone@archlinux.org`

cat pacman-explicits.json | jq --raw-output '
  .[]
    | select( .repository | test("core|extra|multilib") )
    | select( .packager | contains("@archlinux.org") | not)
    | "\(.name), \(.repository), \(.packager)"
'
```
