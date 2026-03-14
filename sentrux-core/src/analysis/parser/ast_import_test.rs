//! Diagnostic test: dumps full tree-sitter AST for import statements across languages.
//!
//! Run with: cargo test ast_import_dump -- --ignored --nocapture
//! This prints the exact node structure tree-sitter produces, so we can build
//! a generic AST walker that extracts module paths WITHOUT text re-parsing.

#[cfg(test)]
mod tests {
    use crate::analysis::lang_registry;
    use tree_sitter::Parser;

    /// Sample import statements per language.
    const SAMPLES: &[(&str, &str)] = &[
        (
            "python",
            r#"import os
import os.path
from collections import OrderedDict
from ..utils import helper
from typing import Protocol, ABC
"#,
        ),
        (
            "rust",
            r#"use std::collections::HashMap;
use crate::models::{episode::{Episode, Injection}, primitive};
mod graph;
pub use self::graph::compute_levels;
"#,
        ),
        (
            "go",
            r#"package main
import (
    "fmt"
    "os"
    cfg "github.com/user/repo/config"
    _ "github.com/lib/pq"
)
"#,
        ),
        (
            "javascript",
            r#"import React from 'react';
import { useState, useEffect } from 'react';
const path = require('path');
"#,
        ),
        (
            "typescript",
            r#"import { Component } from '@angular/core';
import type { Config } from './config';
import * as fs from 'fs';
"#,
        ),
        (
            "java",
            r#"import com.example.UserService;
import static java.util.Collections.emptyList;
import java.util.*;
"#,
        ),
        (
            "c",
            r#"#include <stdio.h>
#include "mylib.h"
#include "../utils/helper.h"
"#,
        ),
        (
            "ruby",
            r#"require 'json'
require_relative './helper'
require_relative '../utils/parser'
"#,
        ),
    ];

    /// Recursively walk and print every node in the tree.
    fn print_tree(
        node: tree_sitter::Node,
        source: &[u8],
        lang: &str,
        depth: usize,
        field_name: Option<&str>,
    ) {
        let indent = "  ".repeat(depth);
        let kind = node.kind();
        let is_named = node.is_named();
        let start = node.start_byte();
        let end = node.end_byte();
        let text_raw = &source[start..end];
        // Truncate long text to keep output readable
        let text = std::str::from_utf8(text_raw).unwrap_or("<non-utf8>");
        let text_display = if text.len() > 80 {
            format!("{}...", &text[..77])
        } else {
            text.replace('\n', "\\n")
        };

        let field_str = match field_name {
            Some(f) => format!(" field:{}", f),
            None => String::new(),
        };
        let named_str = if is_named { "" } else { " [anon]" };

        println!(
            "{}{}  ({}-{}){}{} {:?}",
            indent, kind, start, end, field_str, named_str, text_display,
        );

        // Recurse into children, preserving field names
        let child_count = node.child_count();
        for i in 0..child_count {
            let child = node.child(i).unwrap();
            let child_field = node.field_name_for_child(i as u32);
            print_tree(child, source, lang, depth + 1, child_field);
        }
    }

    #[test]
    #[ignore]
    fn ast_import_dump() {
        let mut parser = Parser::new();
        let mut found_any = false;

        for &(lang, source) in SAMPLES {
            println!("\n{}", "=".repeat(72));
            println!("[{}] Attempting to parse import statements", lang);
            println!("{}", "=".repeat(72));

            let config = match lang_registry::get(lang) {
                Some(c) => c,
                None => {
                    println!("[{}] SKIPPED — plugin not installed", lang);
                    continue;
                }
            };

            found_any = true;

            if let Err(e) = parser.set_language(&config.grammar) {
                println!("[{}] ERROR setting language: {}", lang, e);
                continue;
            }

            let tree = match parser.parse(source.as_bytes(), None) {
                Some(t) => t,
                None => {
                    println!("[{}] ERROR: parser.parse returned None", lang);
                    continue;
                }
            };

            println!("[{}] Source:", lang);
            for (i, line) in source.lines().enumerate() {
                println!("  {:3}| {}", i + 1, line);
            }
            println!();
            println!("[{}] Full AST:", lang);
            print_tree(tree.root_node(), source.as_bytes(), lang, 0, None);
            println!();
        }

        if !found_any {
            println!(
                "\nWARNING: No language plugins were loaded. \
                 Install plugins with `sentrux plugin add-standard`."
            );
        }
    }
}
