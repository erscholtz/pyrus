lexer is simple, only has a buffer and its cursor

lexer moves cursor depending on how far its lexed,

reason for a buffer:

peek_next()
  lex current, store in buffer,
  lex next, store in buffer,
  check last of buffer thats your next

same for current
  lex current, store in buffer
  check last of buffer,

then on pull()
  pulls out of front of buffer
