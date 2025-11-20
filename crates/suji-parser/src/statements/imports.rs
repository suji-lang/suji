use crate::{ParseError, ParseResult, Parser};
use suji_ast::Stmt;
use suji_ast::{ExportBody, ExportSpec};
use suji_lexer::Token;

impl Parser {
    /// Parse import statement: import spec
    pub(super) fn parse_import_statement(&mut self) -> ParseResult<Stmt> {
        let span = self.previous().span.clone();

        if let Token::Identifier(module_name) = &self.peek().token {
            let module_name = module_name.clone();
            let module_span = self.advance().span.clone();

            // Check for colon (import module:item or module:submodule:item)
            if self.check(Token::Colon) {
                // Parse colon-separated path segments, requiring at least one additional segment
                // after the first (module) name.
                let (segments, _path_span) =
                    self.parse_colon_path_from(module_name.clone(), module_span, true)?;
                let (module_path, item_name) = segments
                    .split_last()
                    .map(|(last, rest)| (rest.join(":"), last.to_string()))
                    .unwrap_or_default();

                // Check for 'as' alias
                if self.match_token(Token::As) {
                    let (alias, _alias_span) = match self.consume_identifier() {
                        Ok(v) => v,
                        Err(_) => {
                            let current = self.peek();
                            return Err(ParseError::InvalidAlias { span: current.span });
                        }
                    };
                    Ok(Stmt::Import {
                        spec: suji_ast::ImportSpec::ItemAs {
                            module: module_path,
                            name: item_name,
                            alias,
                        },
                        span,
                    })
                } else {
                    // import module:item
                    Ok(Stmt::Import {
                        spec: suji_ast::ImportSpec::Item {
                            module: module_path,
                            name: item_name,
                        },
                        span,
                    })
                }
            } else {
                // import module
                Ok(Stmt::Import {
                    spec: suji_ast::ImportSpec::Module { name: module_name },
                    span,
                })
            }
        } else {
            Err(ParseError::Generic {
                message: "Expected module name after import".to_string(),
            })
        }
    }

    /// Parse export statement: export { name: expr, ... } | export <expr>
    pub(super) fn parse_export_statement(&mut self) -> ParseResult<Stmt> {
        let span = self.previous().span.clone();

        // Enforce single export per file
        if self.export_count > 0 {
            return Err(ParseError::MultipleExports { span });
        }
        self.export_count += 1;

        // Map form: export { ... }
        if self.match_token(Token::LeftBrace) {
            let mut exports = Vec::new();

            while !self.check(Token::RightBrace) && !self.is_at_end() {
                let (name, _name_span) = self.consume_identifier()?;
                self.consume(Token::Colon, "Expected ':' after export name")?;
                let value = self.expression()?;
                exports.push((name, value));

                if !self.match_token(Token::Comma) {
                    break;
                }
                if self.check(Token::RightBrace) {
                    break;
                }
            }

            self.consume(Token::RightBrace, "Expected '}' after exports")?;

            return Ok(Stmt::Export {
                body: ExportBody::Map(ExportSpec {
                    items: exports,
                    span: span.clone(),
                }),
                span,
            });
        }

        // Expression form: export <expr>
        // If no expression follows, surface a clearer error
        if self.is_at_end() {
            return Err(ParseError::Generic {
                message: "Expected '{' or expression after export".to_string(),
            });
        }
        let expr = self.expression()?;
        Ok(Stmt::Export {
            body: ExportBody::Expr(expr),
            span,
        })
    }
}
