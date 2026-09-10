Parser is smart part, state machine and all that.
- Dumb lexer only parses until a condition is met.
- Even Dumber cursor, lexer just pushes it around.
```text
Parser
  |
  | request token
  v
Lexer.next()
  |
  +-- checkpoint cursor
  |
  +-- pull()
        |
        +-- success ----------------> return Token
        |
        +-- failure
              |
              +-- restore checkpoint
              |
              +-- skip invalid input
              |
              +-- return Diagnostic
                         |
                         v
                 next request continues
```
