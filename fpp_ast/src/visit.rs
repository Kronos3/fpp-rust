/// Each method of the `Visitor` trait is a hook to be potentially
/// overridden. Each method's default implementation recursively visits
/// the substructure of the input via the corresponding `walk` method;
/// e.g., the `visit_item` method by default calls `visit::walk_item`.
///
/// If you want to ensure that your code handles every variant
/// explicitly, you need to override each method. (And you also need
/// to monitor future changes to `Visitor` in case a new method with a
/// new default implementation gets introduced.)
pub trait Visitor: Sized {
    type Break;
    // fn visit_ty(&mut self, ty: &Ty) -> ControlFlow<Self::Break> {
    //     ty.super_visit(self)
    // }
    // fn visit_const(&mut self, c: &TyConst) -> ControlFlow<Self::Break> {
    //     c.super_visit(self)
    // }
    // fn visit_reg(&mut self, reg: &Region) -> ControlFlow<Self::Break> {
    //     reg.super_visit(self)
    // }
}
