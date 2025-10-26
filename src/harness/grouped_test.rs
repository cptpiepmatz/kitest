use std::marker::PhantomData;

use crate::{
    filter::TestFilter,
    group::{TestGroupRunner, TestGroups},
    ignore::TestIgnore,
    panic_handler::TestPanicHandler,
    runner::TestRunner,
    test::Test,
};

pub struct GroupedTestHarness<
    't,
    Extra,
    GroupKey,
    GroupCtx,
    Filter,
    Grouper,
    Groups,
    Ignore,
    GroupRunner,
    PanicHandler,
    Runner,
    Formatter,
> {
    pub(crate) tests: &'t [Test<Extra>],
    pub(crate) _group_key: PhantomData<GroupKey>,
    pub(crate) _group_ctx: PhantomData<GroupCtx>,
    pub(crate) filter: Filter,
    pub(crate) grouper: Grouper,
    pub(crate) groups: Groups,
    pub(crate) ignore: Ignore,
    pub(crate) group_runner: GroupRunner,
    pub(crate) panic_handler: PanicHandler,
    pub(crate) runner: Runner,
    pub(crate) formatter: Formatter,
}

impl<
    't,
    Extra,
    GroupKey,
    GroupCtx,
    Filter,
    Grouper,
    Groups,
    Ignore,
    GroupRunner,
    PanicHandler,
    Runner,
    Formatter,
>
    GroupedTestHarness<
        't,
        Extra,
        GroupKey,
        GroupCtx,
        Filter,
        Grouper,
        Groups,
        Ignore,
        GroupRunner,
        PanicHandler,
        Runner,
        Formatter,
    >
{
    pub fn run(self) {}
}

impl<
    't,
    Extra,
    GroupKey,
    GroupCtx,
    Filter,
    Grouper,
    Groups,
    Ignore,
    GroupRunner,
    PanicHandler,
    Runner,
    Formatter,
>
    GroupedTestHarness<
        't,
        Extra,
        GroupKey,
        GroupCtx,
        Filter,
        Grouper,
        Groups,
        Ignore,
        GroupRunner,
        PanicHandler,
        Runner,
        Formatter,
    >
{
    pub fn list(self) {}
}

impl<
    't,
    Extra,
    GroupKey,
    GroupCtx,
    Filter,
    Grouper,
    Groups,
    Ignore,
    GroupRunner,
    PanicHandler,
    Runner,
    Formatter,
>
    GroupedTestHarness<
        't,
        Extra,
        GroupKey,
        GroupCtx,
        Filter,
        Grouper,
        Groups,
        Ignore,
        GroupRunner,
        PanicHandler,
        Runner,
        Formatter,
    >
{
    pub fn with_filter<WithFilter: TestFilter<Extra>>(
        self,
        filter: WithFilter,
    ) -> GroupedTestHarness<
        't,
        Extra,
        GroupKey,
        GroupCtx,
        WithFilter,
        Grouper,
        Groups,
        Ignore,
        GroupRunner,
        PanicHandler,
        Runner,
        Formatter,
    > {
        GroupedTestHarness {
            tests: self.tests,
            _group_key: PhantomData,
            _group_ctx: PhantomData,
            filter,
            grouper: self.grouper,
            groups: self.groups,
            ignore: self.ignore,
            group_runner: self.group_runner,
            panic_handler: self.panic_handler,
            runner: self.runner,
            formatter: self.formatter,
        }
    }

    pub fn with_groups<WithGroups: TestGroups<'t, Extra, GroupKey>>(
        self,
        groups: WithGroups,
    ) -> GroupedTestHarness<
        't,
        Extra,
        GroupKey,
        GroupCtx,
        Filter,
        Grouper,
        WithGroups,
        Ignore,
        GroupRunner,
        PanicHandler,
        Runner,
        Formatter,
    > {
        GroupedTestHarness {
            tests: self.tests,
            _group_key: PhantomData,
            _group_ctx: PhantomData,
            filter: self.filter,
            grouper: self.grouper,
            groups,
            ignore: self.ignore,
            group_runner: self.group_runner,
            panic_handler: self.panic_handler,
            runner: self.runner,
            formatter: self.formatter,
        }
    }

    pub fn with_ignore<WithIgnore: TestIgnore<Extra>>(
        self,
        ignore: WithIgnore,
    ) -> GroupedTestHarness<
        't,
        Extra,
        GroupKey,
        GroupCtx,
        Filter,
        Grouper,
        Groups,
        WithIgnore,
        GroupRunner,
        PanicHandler,
        Runner,
        Formatter,
    > {
        GroupedTestHarness {
            tests: self.tests,
            _group_key: PhantomData,
            _group_ctx: PhantomData,
            filter: self.filter,
            grouper: self.grouper,
            groups: self.groups,
            ignore,
            group_runner: self.group_runner,
            panic_handler: self.panic_handler,
            runner: self.runner,
            formatter: self.formatter,
        }
    }

    pub fn with_group_runner<WithGroupRunner: TestGroupRunner<Extra, GroupKey, GroupCtx>>(
        self,
        group_runner: WithGroupRunner,
    ) -> GroupedTestHarness<
        't,
        Extra,
        GroupKey,
        GroupCtx,
        Filter,
        Grouper,
        Groups,
        Ignore,
        WithGroupRunner,
        PanicHandler,
        Runner,
        Formatter,
    > {
        GroupedTestHarness {
            tests: self.tests,
            _group_key: PhantomData,
            _group_ctx: PhantomData,
            filter: self.filter,
            grouper: self.grouper,
            groups: self.groups,
            ignore: self.ignore,
            group_runner,
            panic_handler: self.panic_handler,
            runner: self.runner,
            formatter: self.formatter,
        }
    }

    pub fn with_panic_handler<WithPanicHandler: TestPanicHandler<Extra>>(
        self,
        panic_handler: WithPanicHandler,
    ) -> GroupedTestHarness<
        't,
        Extra,
        GroupKey,
        GroupCtx,
        Filter,
        Grouper,
        Groups,
        Ignore,
        GroupRunner,
        WithPanicHandler,
        Runner,
        Formatter,
    > {
        GroupedTestHarness {
            tests: self.tests,
            _group_key: PhantomData,
            _group_ctx: PhantomData,
            filter: self.filter,
            grouper: self.grouper,
            groups: self.groups,
            ignore: self.ignore,
            group_runner: self.group_runner,
            panic_handler,
            runner: self.runner,
            formatter: self.formatter,
        }
    }

    pub fn with_runner<WithRunner: TestRunner<Extra>>(
        self,
        runner: WithRunner,
    ) -> GroupedTestHarness<
        't,
        Extra,
        GroupKey,
        GroupCtx,
        Filter,
        Grouper,
        Groups,
        Ignore,
        GroupRunner,
        PanicHandler,
        WithRunner,
        Formatter,
    > {
        GroupedTestHarness {
            tests: self.tests,
            _group_key: PhantomData,
            _group_ctx: PhantomData,
            filter: self.filter,
            grouper: self.grouper,
            groups: self.groups,
            ignore: self.ignore,
            group_runner: self.group_runner,
            panic_handler: self.panic_handler,
            runner,
            formatter: self.formatter,
        }
    }

    pub fn with_formatter<WithFormatter>(
        self,
        formatter: WithFormatter,
    ) -> GroupedTestHarness<
        't,
        Extra,
        GroupKey,
        GroupCtx,
        Filter,
        Grouper,
        Groups,
        Ignore,
        GroupRunner,
        PanicHandler,
        Runner,
        WithFormatter,
    > {
        GroupedTestHarness {
            tests: self.tests,
            _group_key: PhantomData,
            _group_ctx: PhantomData,
            filter: self.filter,
            grouper: self.grouper,
            groups: self.groups,
            ignore: self.ignore,
            group_runner: self.group_runner,
            panic_handler: self.panic_handler,
            runner: self.runner,
            formatter,
        }
    }
}
