# backwards thinking

- [ ] check all of the fiels and mentions for sizing or size, this could be commmon functionality that can be extracted almost right after AST, since its parsed as a enum then just gets carried over
- [ ] I belive there is another case like this for alignment, maybe some others like config or so but to be determined
- [ ] Think of combining elem call and layout call, elem doest really add anything, specific items are called out in layout already??
- [ ]

# forwards thinking
- [ ] for layout I am thinking I do not need to add in the allocate, 
- [ ] think about maybe having saved ranges for overlapping styles instead of saving them to a specific type right now, might also be faster since the entire string of a row is available without loops
``` rust
struct Content {
    text: String,
    bold: Vec<Span>,
    italic: Vec<Span>,
    nerd: Vec<Span>,
}
```
