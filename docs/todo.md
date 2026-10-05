# backwards thinking

- [ ] check all of the fiels and mentions for sizing or size, this could be commmon functionality that can be extracted almost right after AST, since its parsed as a enum then just gets carried over
- [ ] I belive there is another case like this for alignment, maybe some others like config or so but to be determined
- [ ] Think of combining elem call and layout call, elem doest really add anything, specific items are called out in layout already??
- [ ]

# forwards thinking
- [ ] for layout I am thinking I do not need to add in the allocate, 
- [x] also more inscribe to work on already wrapped rows of string, this would be so fast (maybe think about this more because there would have to be a way to split different fonts like nerd font, right now they are in a vec of content which saves that info along strings but I think this can be done better with spans potentially SOA style rather than AOS style)
``` rust
struct Content {
    text: String,
    bold: Vec<Span>,
    italic: Vec<Span>,
    nerd: Vec<Span>,
}
```
