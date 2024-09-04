mod block_statement;
mod break_statement;
mod continue_statement;
mod declaration;
mod for_in_statement;
mod if_statement;
mod labeled_statements;
mod module_declaration;
mod return_statement;
mod statement_vec;
mod switch_statement;
mod throw_statement;
mod try_statement;
mod while_statement;

use crate::{transformer::Transformer, Analyzer};
use oxc::{
  ast::{
    ast::{ExpressionStatement, Statement},
    match_declaration, match_module_declaration,
  },
  span::GetSpan,
};

impl<'a> Analyzer<'a> {
  pub fn exec_statement(&mut self, node: &'a Statement) {
    match node {
      match_declaration!(Statement) => {
        let node = node.to_declaration();
        self.exec_declaration(node, false);
      }
      match_module_declaration!(Statement) => {
        let node = node.to_module_declaration();
        self.exec_module_declaration(node);
      }
      Statement::ExpressionStatement(node) => {
        self.exec_expression(&node.expression);
      }
      Statement::BlockStatement(node) => self.exec_block_statement(node),
      Statement::IfStatement(node) => self.exec_if_statement(node),
      Statement::WhileStatement(node) => self.exec_while_statement(node),
      Statement::ForInStatement(node) => self.exec_for_in_statement(node),
      Statement::SwitchStatement(node) => self.exec_switch_statement(node),
      Statement::BreakStatement(node) => self.exec_break_statement(node),
      Statement::ContinueStatement(node) => self.exec_continue_statement(node),
      Statement::ReturnStatement(node) => self.exec_return_statement(node),
      Statement::LabeledStatement(node) => self.exec_labeled_statement(node),
      Statement::TryStatement(node) => self.exec_try_statement(node),
      Statement::ThrowStatement(node) => self.exec_throw_statement(node),
      Statement::EmptyStatement(_) => {}
      _ => todo!("Stmt at span {:?}", node.span()),
    }
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_statement(&self, node: &'a Statement<'a>) -> Option<Statement<'a>> {
    let span = node.span();
    match node {
      match_declaration!(Statement) => self
        .transform_declaration(node.to_declaration())
        .map(|decl| self.ast_builder.statement_declaration(decl)),
      match_module_declaration!(Statement) => self
        .transform_module_declaration(node.to_module_declaration())
        .map(|decl| self.ast_builder.statement_module_declaration(decl)),
      Statement::ExpressionStatement(node) => {
        let ExpressionStatement { expression, .. } = node.as_ref();
        self
          .transform_expression(expression, false)
          .map(|expr| self.ast_builder.statement_expression(span, expr))
      }
      Statement::BlockStatement(node) => {
        self.transform_block_statement(node).map(|stmt| self.ast_builder.statement_from_block(stmt))
      }
      Statement::IfStatement(node) => self.transform_if_statement(node),
      Statement::WhileStatement(node) => self.transform_while_statement(node),
      Statement::ForInStatement(node) => self.transform_for_in_statement(node),
      Statement::SwitchStatement(node) => self.transform_switch_statement(node),
      Statement::BreakStatement(node) => self.transform_break_statement(node),
      Statement::ContinueStatement(node) => self.transform_continue_statement(node),
      Statement::ReturnStatement(node) => self.transform_return_statement(node),
      Statement::LabeledStatement(node) => self.transform_labeled_statement(node),
      Statement::TryStatement(node) => self.transform_try_statement(node),
      Statement::ThrowStatement(node) => self.transform_throw_statement(node),
      Statement::EmptyStatement(_) => None,
      _ => todo!(),
    }
  }
}
