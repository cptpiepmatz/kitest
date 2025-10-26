use std::marker::PhantomData;

use crate::{
    GroupedTestHarness, TestReport,
    filter::TestFilter,
    formatter::{TestFormatter, TestListFormatter},
    group::{SimpleGroupRunner, TestGroupHashMap, TestGrouper},
    ignore::TestIgnore,
    panic_handler::TestPanicHandler,
    runner::TestRunner,
    test::Test,
};

pub struct TestHarness<'t, Extra, Filter, Ignore, PanicHandler, Runner, Formatter> {
    pub(crate) tests: &'t [Test<Extra>],
    pub(crate) filter: Filter,
    pub(crate) ignore: Ignore,
    pub(crate) panic_handler: PanicHandler,
    pub(crate) runner: Runner,
    pub(crate) formatter: Formatter,
}

impl<
    't,
    Extra,
    Filter: TestFilter<Extra>,
    Ignore: TestIgnore<Extra>,
    PanicHandler: TestPanicHandler<Extra>,
    Runner: TestRunner<Extra>,
    Formatter: TestFormatter<'t, Extra>,
> TestHarness<'t, Extra, Filter, Ignore, PanicHandler, Runner, Formatter>
{
    pub fn run(self) -> TestReport<'t, Formatter::Error> {
        todo!()
    }
}

impl<
    't,
    Extra,
    Filter: TestFilter<Extra>,
    Ignore: TestIgnore<Extra>,
    PanicHandler,
    Runner,
    Formatter: TestListFormatter<'t, Extra>,
> TestHarness<'t, Extra, Filter, Ignore, PanicHandler, Runner, Formatter>
{
    pub fn list(self) -> impl ExactSizeIterator<Item = (&'static str, Formatter::Error)> {
        [todo!()].into_iter()
    }
}

impl<'t, Extra, Filter, Ignore, PanicHandler, Runner, Formatter>
    TestHarness<'t, Extra, Filter, Ignore, PanicHandler, Runner, Formatter>
{
    pub fn with_ignore<WithIgnore: TestIgnore<Extra>>(
        self,
        ignore: WithIgnore,
    ) -> TestHarness<'t, Extra, Filter, WithIgnore, PanicHandler, Runner, Formatter> {
        TestHarness {
            tests: self.tests,
            filter: self.filter,
            ignore,
            panic_handler: self.panic_handler,
            runner: self.runner,
            formatter: self.formatter,
        }
    }

    pub fn with_filter<WithFilter: TestFilter<Extra>>(
        self,
        filter: WithFilter,
    ) -> TestHarness<'t, Extra, WithFilter, Ignore, PanicHandler, Runner, Formatter> {
        TestHarness {
            tests: self.tests,
            filter,
            ignore: self.ignore,
            panic_handler: self.panic_handler,
            runner: self.runner,
            formatter: self.formatter,
        }
    }

    pub fn with_panic_handler<WithPanicHandler: TestPanicHandler<Extra>>(
        self,
        panic_handler: WithPanicHandler,
    ) -> TestHarness<'t, Extra, Filter, Ignore, WithPanicHandler, Runner, Formatter> {
        TestHarness {
            tests: self.tests,
            filter: self.filter,
            ignore: self.ignore,
            panic_handler,
            runner: self.runner,
            formatter: self.formatter,
        }
    }

    pub fn with_runner<WithRunner: TestRunner<Extra>>(
        self,
        runner: WithRunner,
    ) -> TestHarness<'t, Extra, Filter, Ignore, PanicHandler, WithRunner, Formatter> {
        TestHarness {
            tests: self.tests,
            filter: self.filter,
            ignore: self.ignore,
            panic_handler: self.panic_handler,
            runner,
            formatter: self.formatter,
        }
    }

    pub fn with_formatter<WithFormatter>(
        self,
        formatter: WithFormatter,
    ) -> TestHarness<'t, Extra, Filter, Ignore, PanicHandler, Runner, WithFormatter> {
        TestHarness {
            tests: self.tests,
            filter: self.filter,
            ignore: self.ignore,
            panic_handler: self.panic_handler,
            runner: self.runner,
            formatter,
        }
    }

    pub fn with_grouper<WithGrouper: TestGrouper<Extra, GroupKey, GroupCtx>, GroupKey, GroupCtx>(
        self,
        grouper: WithGrouper,
    ) -> GroupedTestHarness<
        't,
        Extra,
        GroupKey,
        GroupCtx,
        Filter,
        WithGrouper,
        TestGroupHashMap<'t, Extra, GroupKey>,
        Ignore,
        SimpleGroupRunner,
        PanicHandler,
        Runner,
        Formatter,
    > {
        GroupedTestHarness {
            tests: self.tests,
            _group_key: PhantomData,
            _group_ctx: PhantomData,
            filter: self.filter,
            grouper,
            groups: TestGroupHashMap::default(),
            ignore: self.ignore,
            group_runner: SimpleGroupRunner::default(),
            panic_handler: self.panic_handler,
            runner: self.runner,
            formatter: self.formatter,
        }
    }
}
