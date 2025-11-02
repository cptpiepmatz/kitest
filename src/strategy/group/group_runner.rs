pub trait TestGroupRunner<Extra, GroupKey, GroupCtx> {
    fn run_group<F, T>(&self, f: F, key: &GroupKey, ctx: Option<&GroupCtx>) -> T
    where
        F: FnOnce() -> T;
}

#[derive(Debug, Default)]
pub struct SimpleGroupRunner;

impl<Extra, GroupKey, GroupCtx> TestGroupRunner<Extra, GroupKey, GroupCtx> for SimpleGroupRunner {
    fn run_group<F, T>(&self, f: F, _: &GroupKey, _: Option<&GroupCtx>) -> T
    where
        F: FnOnce() -> T,
    {
        f()
    }
}
