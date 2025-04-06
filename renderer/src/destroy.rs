pub trait Destroy<C> {
    fn destroy_with(&mut self, ctx: &C);
}

impl<C, T: Destroy<C>, const N: usize> Destroy<C> for [T; N] {
    fn destroy_with(&mut self, ctx: &C) {
        for e in self {
            e.destroy_with(ctx);
        }
    }
}

impl<C, T: Destroy<C>> Destroy<C> for Vec<T> {
    fn destroy_with(&mut self, ctx: &C) {
        for e in self {
            e.destroy_with(ctx);
        }
    }
}
