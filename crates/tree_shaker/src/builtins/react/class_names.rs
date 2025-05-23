use crate::{
  analyzer::Factory, builtins::prototypes::BuiltinPrototypes, entity::Entity, value::TypeofResult,
};

pub fn create_class_names_namespace<'a>(
  factory: &'a Factory<'a>,
  _prototypes: &'a BuiltinPrototypes<'a>,
) -> Entity<'a> {
  factory.implemented_builtin_fn("classnames::default", |analyzer, dep, _this, args| {
    let (class_names, rest, iterate_dep) = args.iterate(analyzer, dep);

    let mut deps_1 = factory.vec();
    let mut deps_2 = factory.vec1(iterate_dep);
    for class_name in class_names {
      if TypeofResult::Object.contains(class_name.test_typeof()) {
        // This may be an array. However, this makes no difference in this logic.
        let enumerated = class_name.enumerate_properties(analyzer, dep);
        for (_, key, value) in enumerated.known.into_values() {
          if value.test_truthy() != Some(false) {
            deps_1.push(key);
            deps_1.push(value);
          }
        }
        if let Some(u) = enumerated.unknown {
          deps_1.push(u);
        }
        deps_2.push(enumerated.dep);
      } else {
        deps_1.push(class_name);
      }
    }

    analyzer.factory.computed_unknown_string((deps_1, deps_2, rest))
  })
}
