; Bash structural queries

; Functions
(function_definition
  name: (word) @func.name) @func.def

; Commands (calls)
(command
  name: (command_name) @call.name) @call

; ---- Import appendix (custom) ----

; source ./file.sh / . ./file.sh
(command
  name: (command_name) @_cmd
  argument: (word) @import.module
  (#match? @_cmd "^(source|\\.)$")) @import
