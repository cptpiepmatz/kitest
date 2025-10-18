use kitest::{TestExecutor, TestFn, TestMeta};
use std::borrow::Cow;

static TESTS: &[TestMeta] = &[TestMeta {
    name: Cow::Borrowed("some::example_ok"),
    function: TestFn::plain(some::example_ok),
    ignored: (false, None),
    should_panic: (false, None),
    extra: (),
}];

mod some {
    pub fn example_ok() {}
}

fn main() {
    TestExecutor::new(TESTS).run(|meta| meta.run()).exit();
}
