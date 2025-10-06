use super::{ParseError, ParseResult, Parser};
use suji_ast::ast::Stmt;
use suji_lexer::token::Token;

impl Parser {
    /// Parse import statement: import spec
    pub(super) fn parse_import_statement(&mut self) -> ParseResult<Stmt> {
        let span = self.previous().span.clone();

        if let Token::Identifier(module_name) = &self.peek().token {
            let module_name = module_name.clone();
            self.advance();

            // Check for colon (import module:item or module:submodule:item)
            if self.match_token(Token::Colon) {
                // Parse the module path (can be nested like "json:parse" or just "parse")
                let mut module_path = module_name.clone();

                // Keep parsing identifiers separated by colons
                while let Token::Identifier(part) = &self.peek().token {
                    let part = part.clone();
                    self.advance();
                    module_path.push(':');
                    module_path.push_str(&part);

                    if !self.match_token(Token::Colon) {
                        break;
                    }
                }

                // Check if we have at least one item after the colon
                if module_path == module_name {
                    return Err(ParseError::Generic {
                        message: "Expected item name after ':'".to_string(),
                    });
                }

                // The last part is the item name
                let item_name = module_path.split(':').next_back().unwrap().to_string();
                let module_path = module_path
                    .trim_end_matches(&format!(":{}", item_name))
                    .to_string();

                // Check for 'as' alias
                if self.match_token(Token::As) {
                    if let Token::Identifier(alias) = &self.peek().token {
                        let alias = alias.clone();
                        self.advance();

                        Ok(Stmt::Import {
                            spec: suji_ast::ast::ImportSpec::ItemAs {
                                module: module_path,
                                name: item_name,
                                alias,
                            },
                            span,
                        })
                    } else {
                        Err(ParseError::Generic {
                            message: "Expected alias name after 'as'".to_string(),
                        })
                    }
                } else {
                    // import module:item
                    Ok(Stmt::Import {
                        spec: suji_ast::ast::ImportSpec::Item {
                            module: module_path,
                            name: item_name,
                        },
                        span,
                    })
                }
            } else {
                // import module
                Ok(Stmt::Import {
                    spec: suji_ast::ast::ImportSpec::Module { name: module_name },
                    span,
                })
            }
        } else {
            Err(ParseError::Generic {
                message: "Expected module name after import".to_string(),
            })
        }
    }

    /// Parse export statement: export { name: expr, ... }
    pub(super) fn parse_export_statement(&mut self) -> ParseResult<Stmt> {
        let span = self.previous().span.clone();

        // Enforce single export per file
        if self.export_count > 0 {
            return Err(ParseError::MultipleExports { span });
        }
        self.export_count += 1;

        self.consume(Token::LeftBrace, "Expected '{' after export")?;
        let mut exports = Vec::new();

        while !self.check(Token::RightBrace) && !self.is_at_end() {
            if let Token::Identifier(name) = &self.peek().token {
                let name = name.clone();
                self.advance();
                self.consume(Token::Colon, "Expected ':' after export name")?;
                let value = self.expression()?;
                exports.push((name, value));

                if !self.match_token(Token::Comma) {
                    break;
                }
                if self.check(Token::RightBrace) {
                    break;
                }
            } else {
                return Err(ParseError::Generic {
                    message: "Expected export name".to_string(),
                });
            }
        }

        self.consume(Token::RightBrace, "Expected '}' after exports")?;

        Ok(Stmt::Export {
            spec: suji_ast::ast::ExportSpec {
                items: exports,
                span: span.clone(),
            },
            span,
        })
    }
}
