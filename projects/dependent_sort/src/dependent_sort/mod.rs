use std::cmp::Ordering;
use std::collections::{ HashMap};
use std::collections::hash_map::Entry;
use std::hash::Hash;
use std::ops::AddAssign;
use itertools::Itertools;

mod arithmetic;

#[derive(Default)]
pub struct DependentSort<'i, T, G> {
    tasks: Vec<Task<'i, T, G>>,
    /// Should circular dependencies report an error immediately or declare them at the same time?
    allow_circular: bool,
}

/// Every task must belong to a group
/// Ungrouped tasks are assigned a virtual number
#[derive(Debug)]
struct VirtualGroup<'i, T> {
    id: usize,
    dependent_groups: Vec<usize>,
    contain_tasks: Vec<&'i T>,
}

#[derive(Debug)]
pub struct Task<'i, T, G> {
    pub id: &'i T,
    pub group: Option<&'i G>,
    pub dependent_tasks: Vec<&'i T>,
}


impl<'i, T, G> DependentSort<'i, T, G> {
    /// - First assign a group number to each grouped task
    /// - Assign auto-increment ids to tasks without groups
    fn virtual_groups(&self) -> Vec<VirtualGroup<'i, T>>
        where
            T: Eq + Ord,
            G: Eq + Ord + Hash,
    {
        let reals = self.tasks.iter().filter(|task| task.group.is_some()).dedup().count();
        let mut groups = Vec::with_capacity(self.tasks.len() - reals);
        let mut real_id = 0;
        let mut virtual_id = reals + 1;
        let mut group_map = HashMap::new();
        for task in self.tasks.iter() {
            match task.group {
                Some(group) => {
                    match group_map.entry(group) {
                        Entry::Occupied( entry) => {
                            let id = *entry.get();
                            let group: &mut VirtualGroup<T> = groups.get_mut(id).unwrap();
                            group.contain_tasks.push(task.id);
                        }
                        Entry::Vacant(entry) => {
                            let group = VirtualGroup { id: real_id, dependent_groups: vec![], contain_tasks: vec![task.id] };
                            groups.push(group);
                            entry.insert(real_id);
                            real_id += 1;
                        }
                    }
                }
                None => { continue; }
            }
        }
        for task in self.tasks.iter().filter(|task| task.group.is_none()) {
            let group = VirtualGroup { id: virtual_id, dependent_groups: vec![], contain_tasks: vec![task.id] };
            groups.push(group);
            virtual_id += 1;
        }
        groups
    }
    // 1.定义虚拟组
    // 2.给虚拟组拓扑排序
    // 3.每个虚拟组内部拓扑排序
    // 4.按照虚拟组的顺序输出任务
    pub fn sort(&self) -> Vec<T>
        where
            T: Clone + Eq + Hash,
            G: Clone + Eq + Hash,
    {
        todo!()
    }
}


impl<'i, T, G> Task<'i, T, G> {
    pub fn new(id: &'i T) -> Self {
        Self { id, group: None, dependent_tasks: vec![] }
    }
    pub fn new_with_dependent(id: &'i T, dependent_tasks: Vec<&'i T>) -> Self {
        Self { id, group: None, dependent_tasks }
    }
    pub fn with_group(self, group: &'i G) -> Self {
        Self { group: Some(group), ..self }
    }
}


#[test]
fn test() {
    let mut tasks = DependentSort::default();
    tasks += Task::new(&0);
    tasks += Task::new_with_dependent(&1, vec![&6]);
    tasks += Task::new_with_dependent(&2, vec![&5]).with_group(&"A");
    tasks += Task::new_with_dependent(&3, vec![&6]).with_group(&"B");
    tasks += Task::new_with_dependent(&4, vec![&3, &6]).with_group(&"B");
    tasks += Task::new_with_dependent(&5, vec![]).with_group(&"A");
    tasks += Task::new_with_dependent(&6, vec![]).with_group(&"B");
    tasks += Task::new(&7);

    let groups = tasks.virtual_groups();
    println!("{:#?}", groups);

    // let sorted = tasks.sort();
    // assert_eq!(sorted, vec![6, 3, 4, 1, 5, 2, 0, 7]);
}
