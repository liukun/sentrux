; Nim tags.scm — verified against actual AST

; functions: routine → symbol → ident
(routine
  (symbol
    (ident) @name)) @definition.function

; types: typeDef → symbol → ident
(typeDef
  (symbol
    (ident) @name)) @definition.class

; imports: importStmt → expr (read text as module path)
(importStmt
  (expr) @import.module) @import
(fromStmt
  (expr) @import.module) @import
(includeStmt
  (expr) @import.module) @import

; ── Calls ──
; Function call: procName(args) — primary contains symbol → ident, then primarySuffix(functionCall)
(primarySuffix
  (functionCall)) @reference.call

; Command-style call: echo "hello"
(cmdCall
  (expr
    (primary
      (symbol
        (ident) @name)))) @reference.call

; ── Type references ──
; Type usage in parameter type annotations
(paramColonEquals
  (symbol
    (ident) @name)) @reference.type
