| symbol | meaning                 |
| ------ | ----------------------- |
| `<`    | justify left            |
| `>`    | justify right           |
| `\|`   | split                   |
| `<<`   | override justify left   |
| `>>`   | override justify right  |


The reason for the override situation is because the justify right could have 
this effect:

```
some text                                                            other text
                                                                     more
                                                                     more
```

then you would use the override to get this effect:

```
some text                                                            other text
                                                                           more
                                                                           more
```


| Stage | Owns | Returns |
|---|---|---|
| Inscribe | What content is this, with what typography? | Created Glyphs, GlyphRow |
| Allocate |  How much horizontal space does each field get? | ContentRow |
| Wrap |  Which content fits on each line? | WrappedRow(s) |
| Compose | How do those lines sit together in a band? | ResolvedRow(s) |
| Paginate | On which page, and where, does each band go? | ResolvedPage(s) |
| Doc | Stiches pages together and adds them to a doc | ResolvedDocument |

then render, i guess
