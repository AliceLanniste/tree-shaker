use oxc::{
  ast::ast::{
    Expression, ObjectExpression, ObjectProperty, ObjectPropertyKind, PropertyKind, SpreadElement,
  },
  span::{GetSpan, SPAN},
};

use crate::{
  analyzer::Analyzer, ast::AstKind2, build_effect, entity::Entity, transformer::Transformer,
  value::ObjectPrototype,
};

impl<'a> Analyzer<'a> {
  pub fn exec_object_expression(&mut self, node: &'a ObjectExpression) -> Entity<'a> {
    let object = self.use_mangable_plain_object(AstKind2::ObjectExpression(node));

    for property in &node.properties {
      match property {
        ObjectPropertyKind::ObjectProperty(node) => {
          let key = self.exec_property_key(&node.key);
          let value = self.exec_expression(&node.value);
          if node.key.is_specific_id("__proto__") {
            let dep = self.dep((key, value, AstKind2::ObjectProperty(node)));
            object.set_prototype(if value.test_nullish() == Some(true) {
              ObjectPrototype::ImplicitOrNull
            } else {
              ObjectPrototype::Unknown(dep)
            });
            object.add_extra_dep(dep);
          } else {
            let value = self.factory.computed(value, AstKind2::ObjectProperty(node));
            object.init_property(self, node.kind, key, value, true);
          }
        }
        ObjectPropertyKind::SpreadProperty(node) => {
          let argument = self.exec_expression(&node.argument);
          object.init_spread(self, AstKind2::SpreadElement(node), argument);
        }
      }
    }

    object.into()
  }
}

impl<'a> Transformer<'a> {
  pub fn transform_object_expression(
    &self,
    node: &'a ObjectExpression<'a>,
    need_val: bool,
  ) -> Option<Expression<'a>> {
    let ObjectExpression { span, properties, .. } = node;

    if need_val {
      let mut transformed_properties = self.ast_builder.vec();
      for property in properties {
        transformed_properties.push(match property {
          ObjectPropertyKind::ObjectProperty(node) => {
            let ObjectProperty { span, key, kind, value, method, computed, .. } = node.as_ref();

            let value_span = value.span();

            let transformed_value =
              self.transform_expression(value, self.is_referred(AstKind2::ObjectProperty(node)));

            if let Some(mut transformed_value) = transformed_value {
              if *kind == PropertyKind::Set {
                if let (
                  Expression::FunctionExpression(original_node),
                  Expression::FunctionExpression(transformed_node),
                ) = (value, &mut transformed_value)
                {
                  self.patch_method_definition_params(original_node, transformed_node);
                } else {
                  unreachable!()
                }
              }

              let key = self.transform_property_key(key, true).unwrap();
              self.ast_builder.object_property_kind_object_property(
                *span,
                *kind,
                key,
                transformed_value,
                *method,
                false,
                *computed,
              )
            } else if let Some(key) = self.transform_property_key(key, false) {
              self.ast_builder.object_property_kind_object_property(
                *span,
                *kind,
                key,
                self.build_unused_expression(value_span),
                *method,
                false,
                *computed,
              )
            } else {
              continue;
            }
          }
          ObjectPropertyKind::SpreadProperty(node) => {
            let SpreadElement { span, argument } = node.as_ref();

            let referred = self.is_referred(AstKind2::SpreadElement(node));

            let argument = self.transform_expression(argument, referred);

            if let Some(argument) = argument {
              self.ast_builder.object_property_kind_spread_property(
                *span,
                if referred {
                  argument
                } else {
                  build_effect!(
                    &self.ast_builder,
                    *span,
                    Some(argument);
                    self.ast_builder.expression_object(SPAN, self.ast_builder.vec(), None)
                  )
                },
              )
            } else {
              continue;
            }
          }
        });
      }
      Some(self.ast_builder.expression_object(*span, transformed_properties, None))
    } else {
      let mut effects = vec![];
      for property in properties {
        match property {
          ObjectPropertyKind::ObjectProperty(node) => {
            let ObjectProperty { key, value, .. } = node.as_ref();

            if let Some(key) = self.transform_property_key(key, false) {
              if let Ok(key) = key.try_into() {
                effects.push(key);
              }
            }
            if let Some(value) = self.transform_expression(value, false) {
              effects.push(value);
            }
          }
          ObjectPropertyKind::SpreadProperty(node) => {
            let SpreadElement { span, argument } = node.as_ref();

            let need_spread = self.is_referred(AstKind2::SpreadElement(node));
            if let Some(argument) = self.transform_expression(argument, need_spread) {
              effects.push(if need_spread {
                self.build_object_spread_effect(*span, argument)
              } else {
                argument
              });
            }
          }
        }
      }
      build_effect!(&self.ast_builder, *span, effects)
    }
  }
}
