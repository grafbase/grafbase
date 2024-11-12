use grafbase_workspace_hack as _;

mod iter;

pub use iter::*;

pub trait Iter: ExactSizeIterator + std::iter::FusedIterator + DoubleEndedIterator + std::fmt::Debug {}
impl<T: ExactSizeIterator + std::iter::FusedIterator + DoubleEndedIterator + std::fmt::Debug> Iter for T {}

pub trait Walk<Ctx> {
    type Walker<'a>
    where
        Self: 'a,
        Ctx: 'a;

    fn walk<'a>(self, ctx: impl Into<Ctx>) -> Self::Walker<'a>
    where
        Self: 'a,
        Ctx: 'a;
}

impl<Ctx> Walk<Ctx> for () {
    type Walker<'a> = () where Ctx: 'a;
    fn walk<'a>(self, _: impl Into<Ctx>) -> Self::Walker<'a>
    where
        Self: 'a,
        Ctx: 'a,
    {
    }
}

pub type Walker<'a, T, G> = <T as Walk<G>>::Walker<'a>;

// Hint: Go to the definition of T instead to find the Walker type.
//       Walk implementations are always close to T.
//
/// Convenient blanket implementation to write:
/// `id.read(schema)` rather than `(*id).read(schema)`
impl<Ctx, T: Copy + Walk<Ctx>> Walk<Ctx> for &T {
    type Walker<'a> = Walker<'a, T, Ctx>
    where
        Self: 'a,
        Ctx: 'a;

    fn walk<'a>(self, ctx: impl Into<Ctx>) -> Self::Walker<'a>
    where
        Self: 'a,
        Ctx: 'a,
    {
        (*self).walk(ctx)
    }
}

// Hint: Go to the definition of T instead to find the Walker type.
//       Walk implementations are always close to T.
impl<Ctx, T: Walk<Ctx>> Walk<Ctx> for Option<T> {
    type Walker<'a> = Option<Walker<'a, T, Ctx>>
    where
        Self: 'a,
        Ctx: 'a;

    fn walk<'a>(self, ctx: impl Into<Ctx>) -> Self::Walker<'a>
    where
        Self: 'a,
        Ctx: 'a,
    {
        self.map(|value| value.walk(ctx))
    }
}

// Hint: Go to the definition of T instead to find the Walker type.
//       Walk implementations are always close to T.
impl<Ctx, T> Walk<Ctx> for &[T]
where
    for<'a> &'a T: Walk<Ctx>,
    Ctx: Copy,
{
    type Walker<'a> = WalkIterator<'a, std::slice::Iter<'a, T>, Ctx>
    where
        Self: 'a,
        Ctx: 'a;

    fn walk<'a>(self, ctx: impl Into<Ctx>) -> Self::Walker<'a>
    where
        Self: 'a,
        Ctx: 'a,
    {
        WalkIterator::new(self.iter(), ctx.into())
    }
}
