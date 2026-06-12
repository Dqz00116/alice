pub trait HasComponent<C> {
    fn get(&self) -> &C;
    fn get_mut(&mut self) -> &mut C;
}

pub struct World<Components> {
    components: Components,
}

impl<Components> World<Components> {
    pub fn new(components: Components) -> Self {
        Self { components }
    }

    pub fn get<C>(&self) -> &C
    where
        Components: HasComponent<C>,
    {
        self.components.get()
    }

    pub fn get_mut<C>(&mut self) -> &mut C
    where
        Components: HasComponent<C>,
    {
        self.components.get_mut()
    }

    pub fn snapshot(&self) -> Snapshot<&Components> {
        Snapshot::new(&self.components)
    }
}

pub struct Snapshot<C> {
    components: C,
}

impl<C> Snapshot<C> {
    pub fn new(components: C) -> Self {
        Self { components }
    }

    pub fn get<Comp>(&self) -> &Comp
    where
        C: HasComponent<Comp>,
    {
        self.components.get()
    }
}
