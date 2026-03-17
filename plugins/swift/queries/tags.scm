; Based on official tree-sitter-swift tags.scm (v0.7.1)
; Modified: removed class_body method patterns (they assign class-level line numbers)
; Using standalone function/init patterns which give correct per-function line numbers.

(class_declaration
  name: (type_identifier) @name) @definition.class

(struct_declaration
  name: (type_identifier) @name) @definition.class

(enum_declaration
  name: (type_identifier) @name) @definition.class

(protocol_declaration
  name: (type_identifier) @name) @definition.interface

(extension_declaration
  (user_type (type_identifier) @name)) @definition.class

(function_declaration
    name: (simple_identifier) @name) @definition.function

; ---- Entry point: @main attribute on struct ----
(attribute (user_type (type_identifier) @_n)) @entry

; ---- Import appendix + calls (custom) ----

; Init declarations
(init_declaration) @func.def

; import Foundation / import UIKit.NSView
(import_declaration
  (identifier) @import.module) @import

; Calls — direct function call
(call_expression
  (simple_identifier) @call.name) @call

; Calls — navigation  object.method()
(call_expression
  (navigation_expression
    (navigation_suffix
      (simple_identifier) @call.name))) @call

; Type references — captures type names used in annotations, inits, generics.
; This is critical for Swift because same-target files don't import each other.
; Without this, sentrux is blind to intra-module dependencies.
; e.g., let x = FEMScaffold() → captures "FEMScaffold"
;       var map: DisplacementMap → captures "DisplacementMap"
;       class Foo: BarProtocol → captures "BarProtocol"
(user_type
  (type_identifier) @reference.type)

; Inheritance — captures base class/protocol names
(inheritance_specifier
  (user_type
    (type_identifier) @reference.type))
