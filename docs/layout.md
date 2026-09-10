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
