; Official tree-sitter-r tags.scm (v1.2.0)

(binary_operator
    lhs: (identifier) @name
    operator: "<-"
    rhs: (function_definition)
) @definition.function

(binary_operator
    lhs: (identifier) @name
    operator: "="
    rhs: (function_definition)
) @definition.function

(binary_operator
    lhs: (string) @name
    operator: "<-"
    rhs: (function_definition)
) @definition.function

(binary_operator
    lhs: (string) @name
    operator: "="
    rhs: (function_definition)
) @definition.function

(call
    function: (identifier) @name
) @reference.call

(call
    function: (namespace_operator
        rhs: (identifier) @name
    )
) @reference.call

; ---- Import appendix (custom) ----

; library("package") / require("package") / source("file.R")
(call
    function: (identifier) @_fn
    arguments: (arguments
        (string) @import.module)
    (#match? @_fn "^(library|require|source)$")) @import
