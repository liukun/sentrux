; COBOL tags.scm — verified against actual AST

; programs: program_definition → identification_division → program_name
(program_definition
  (identification_division
    (program_name) @name)) @definition.function

; imports: COPY statement → string field:book
(copy_statement
  book: (string) @import.module) @import

; ── Calls ──
; CALL statement: CALL "subprogram-name"
(call_statement
  (string) @name) @reference.call

; PERFORM statement: PERFORM paragraph-name
(perform_statement_call_proc
  (perform_procedure
    (label) @name)) @reference.call
