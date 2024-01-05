use super::*;

impl<'i, T, G> Clone for Task<'i, T, G> {
    fn clone(&self) -> Self {
        Self { id: self.id, group: self.group, dependent_tasks: self.dependent_tasks.clone() }
    }
}

impl<'i, T, G> PartialEq for Task<'i, T, G>
    where
        T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(other.id)
    }
}

impl<'i, T, G> PartialOrd for Task<'i, T, G>
    where
        T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.id.partial_cmp(other.id)
    }
}

impl<'i, T, G> Eq for Task<'i, T, G> where
    T: Eq {}

impl<'i, T, G> Ord for Task<'i, T, G> where
    T: Ord, {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(other.id)
    }
}

impl<'i, T, G> AddAssign<Task<'i, T, G>> for DependentSort<'i, T, G> where T: PartialEq, G: PartialEq {
    fn add_assign(&mut self, task: Task<'i, T, G>) {
        self.tasks.push(task)
    }
}