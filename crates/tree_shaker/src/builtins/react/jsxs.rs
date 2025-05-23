use crate::{analyzer::Factory, entity::Entity};

pub fn create_react_jsxs_impl<'a>(factory: &'a Factory<'a>) -> Entity<'a> {
  factory.implemented_builtin_fn("React::jsxs", |analyzer, dep, _this, args| {
    let args = args.destruct_as_array(analyzer, dep, 3, false).0;
    let [tag, props, key] = args[..] else { unreachable!() };
    analyzer.consume(props.get_shallow_dep(analyzer));
    props.set_property(analyzer, analyzer.factory.no_dep, analyzer.factory.string("key"), key);
    analyzer.factory.react_element(tag, props)
  })
}
