/// Runtime-neutral matcher contract over entity IDs for a context `Ctx`.
pub trait EntityMatcher<Ctx> {
    fn matches(&self, ctx: &Ctx, entity: u64) -> bool;
}

/// Composite matcher: `A && B`.
pub struct AndMatcher<A, B>(pub A, pub B);

impl<Ctx, A, B> EntityMatcher<Ctx> for AndMatcher<A, B>
where
    A: EntityMatcher<Ctx>,
    B: EntityMatcher<Ctx>,
{
    fn matches(&self, ctx: &Ctx, entity: u64) -> bool {
        self.0.matches(ctx, entity) && self.1.matches(ctx, entity)
    }
}

/// Domain-neutral filter trait alias over matcher contract.
pub trait QueryFilter<Ctx>: EntityMatcher<Ctx> {}
impl<Ctx, T> QueryFilter<Ctx> for T where T: EntityMatcher<Ctx> {}

/// Domain-neutral alias for matcher composition.
pub type And<A, B> = AndMatcher<A, B>;

/// Domain-neutral alias for filtered entity iteration.
pub type QueryIter<'a, Ctx, F> = FilteredEntityIter<'a, Ctx, F>;

/// Generic iterator over entities matching a matcher against immutable context.
pub struct FilteredEntityIter<'a, Ctx, M>
where
    M: EntityMatcher<Ctx>,
{
    ctx: &'a Ctx,
    entities: std::vec::IntoIter<u64>,
    matcher: M,
}

impl<'a, Ctx, M> FilteredEntityIter<'a, Ctx, M>
where
    M: EntityMatcher<Ctx>,
{
    pub fn new<I>(ctx: &'a Ctx, entities: I, matcher: M) -> Self
    where
        I: IntoIterator<Item = u64>,
    {
        let entities: Vec<u64> = entities.into_iter().collect();
        Self {
            ctx,
            entities: entities.into_iter(),
            matcher,
        }
    }
}

impl<'a, Ctx, M> Iterator for FilteredEntityIter<'a, Ctx, M>
where
    M: EntityMatcher<Ctx>,
{
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let entity = self.entities.next()?;
            if self.matcher.matches(self.ctx, entity) {
                return Some(entity);
            }
        }
    }
}

/// Read-only access wrapper for a component bound to an entity ID.
#[derive(Clone, Copy)]
pub struct ReadComponent<'a, T> {
    pub entity: u64,
    pub component: &'a T,
}

/// Mutable access wrapper for a component bound to an entity ID.
pub struct WriteComponent<'a, T> {
    pub entity: u64,
    pub component: &'a mut T,
}

/// Collect matching entities into a `Vec<u64>`.
pub fn collect_matching<Ctx, M, I>(ctx: &Ctx, entities: I, matcher: M) -> Vec<u64>
where
    M: EntityMatcher<Ctx>,
    I: IntoIterator<Item = u64>,
{
    FilteredEntityIter::new(ctx, entities, matcher).collect()
}

/// Count matching entities.
pub fn count_matching<Ctx, M, I>(ctx: &Ctx, entities: I, matcher: M) -> usize
where
    M: EntityMatcher<Ctx>,
    I: IntoIterator<Item = u64>,
{
    FilteredEntityIter::new(ctx, entities, matcher).count()
}
