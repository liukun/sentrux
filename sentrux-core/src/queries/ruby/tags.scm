; Official tree-sitter-ruby tags.scm (v0.23.1)

; Method definitions
(
  (comment)* @doc
  .
  [
    (method
      name: (_) @name) @definition.method
    (singleton_method
      name: (_) @name) @definition.method
  ]
  (#strip! @doc "^#\\s*")
  (#select-adjacent! @doc @definition.method)
)

(alias
  name: (_) @name) @definition.method

(setter
  (identifier) @ignore)

; Class definitions
(
  (comment)* @doc
  .
  [
    (class
      name: [
        (constant) @name
        (scope_resolution
          name: (_) @name)
      ]) @definition.class
    (singleton_class
      value: [
        (constant) @name
        (scope_resolution
          name: (_) @name)
      ]) @definition.class
  ]
  (#strip! @doc "^#\\s*")
  (#select-adjacent! @doc @definition.class)
)

; Module definitions
(
  (module
    name: [
      (constant) @name
      (scope_resolution
        name: (_) @name)
    ]) @definition.module
)

; Calls
(call method: (identifier) @name) @reference.call

(
  [(identifier) (constant)] @name @reference.call
  (#is-not? local)
  (#not-match? @name "^(lambda|load|require|require_relative|__FILE__|__LINE__)$")
)

; ---- Import appendix (custom) ----

; require 'json' / require_relative './helper'
(call
  method: (identifier) @_method
  arguments: (argument_list
    (string) @import.module)
  (#match? @_method "^(require|require_relative)$")) @import
