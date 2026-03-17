; Zig tags.scm — functions, imports, calls

; ── Definitions ──

(function_declaration
  name: (identifier) @name) @definition.function

; ── Imports ──

(variable_declaration
  (builtin_function
    (builtin_identifier) @_fn
    (arguments (string) @import.module)
    (#eq? @_fn "@import"))) @import

; ── Calls ──

; Direct call: foo()
(call_expression
  function: (identifier) @call.name) @call

; Method/field call: obj.method()
(call_expression
  function: (field_expression
    member: (identifier) @call.name)) @call
