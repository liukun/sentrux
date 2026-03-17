; Nix tags.scm — verified against actual AST

; bindings (nix functions/values)
(binding
  attrpath: (attrpath
    (identifier) @name)) @definition.function

; imports: import ./path.nix → apply_expression(function: "import", argument: path)
(apply_expression
  function: (variable_expression) @_fn
  argument: (path_expression) @import.module
  (#eq? @_fn "import")) @import

; import <nixpkgs> → path in angle brackets
(apply_expression
  function: (variable_expression) @_fn2
  argument: (_) @import.module
  (#eq? @_fn2 "import")) @import

; ── Calls ──
; Function application: func arg (general apply_expression, excluding import)
(apply_expression
  function: (variable_expression) @name
  (#not-eq? @name "import")) @reference.call

; Attribute path call: pkgs.mkShell { }
(apply_expression
  function: (select_expression
    (variable_expression) @name)) @reference.call
