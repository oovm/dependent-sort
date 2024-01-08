use crate::TopologicalError;
use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashMap, VecDeque},
    fmt::{Debug, Display},
    hash::Hash,
    ops::AddAssign,
};

mod arithmetic;
mod display;
// mod mermaid;

/// A topological sort of tasks with dependencies.
#[allow(dead_code)]
#[derive(Debug)]
pub struct DependentSort<'i, T, G> {
    /// non-virtualized tasks
    tasks: Vec<Task<'i, T, G>>,
    // virtual_group_id: isize,
    /// Should circular dependencies report an error immediately or declare them at the same time.
    allow_circular: bool,
}

/// A task or type who has dependencies and optionally belongs to a group.
#[derive(Debug)]
pub struct Task<'i, T, G> {
    /// The unique identifier of the task.
    pub id: &'i T,
    /// The group to which the task belongs.
    pub group: Option<&'i G>,
    /// The tasks that this task depends on.
    pub dependent_tasks: Vec<&'i T>,
}

/// A group of tasks, if task is not in a group, it will be in a group of its own.
#[derive(Debug)]
pub struct Group<'i, T, G> {
    /// The unique identifier of the group.
    pub id: Option<&'i G>,
    /// The tasks that this group contains.
    pub tasks: Vec<&'i T>,
}

#[derive(Debug)]
struct FinalizeDependencies<'i, T, G> {
    /// maps for recovering the original tasks
    task_map: Vec<&'i T>,
    /// maps for recovering the original groups
    group_map: Vec<&'i G>,
    /// virtualized tasks
    virtualized_groups: Vec<isize>,
    virtualized_dependent_tasks: Vec<Vec<usize>>,
}

impl<'i, T, G> Task<'i, T, G> {
    /// Create a new task without dependencies.
    pub fn new(id: &'i T) -> Self {
        Self { id, group: None, dependent_tasks: vec![] }
    }
    /// Create a new task with given dependencies.
    pub fn new_with_dependent(id: &'i T, dependent_tasks: Vec<&'i T>) -> Self {
        Self { id, group: None, dependent_tasks }
    }
    /// Set the group to which the task belongs.
    pub fn with_group(self, group: &'i G) -> Self {
        Self { group: Some(group), ..self }
    }
}

impl<'i, T, G> DependentSort<'i, T, G>
where
    T: PartialEq,
    G: PartialEq,
{
    fn finalize(&self) -> Result<FinalizeDependencies<'i, T, G>, TopologicalError<'i, T, G>> {
        let mut sorter = FinalizeDependencies::default();
        for task in &self.tasks {
            sorter.task_map.push(task.id);
            if let Some(group) = task.group {
                if !sorter.group_map.contains(&group) {
                    sorter.group_map.push(group);
                }
            }
        }
        for task in &self.tasks {
            sorter.virtualize(task)?;
        }
        Ok(sorter)
    }
    /// Sort the tasks and return the sorted tasks.
    pub fn sort(&mut self) -> Result<Vec<Task<'i, T, G>>, TopologicalError<'i, T, G>> {
        let sorter = self.finalize()?;
        let sorted = double_topological_sort(
            sorter.virtualized_groups.clone(),
            sorter.virtualized_groups.len(),
            sorter.virtualized_dependent_tasks.clone(),
        );
        Ok(sorted.into_iter().map(|i| self.tasks[i as usize].clone()).collect())
    }
    /// Sort the tasks and return the sorted tasks grouped by their group.
    pub fn sort_grouped(&mut self) -> Result<Vec<Group<'i, T, G>>, TopologicalError<'i, T, G>>
    where
        G: PartialEq,
    {
        let sorted = self.sort()?;
        let mut group_index_map: Vec<(&G, usize)> = vec![];
        let mut grouped: Vec<Group<T, G>> = vec![];
        'outer: for task in sorted {
            match task.group {
                Some(s) => {
                    for (key, position) in group_index_map.iter().map(|(k, v)| (*k, *v)) {
                        // group has been defined
                        if key == s {
                            grouped[position].tasks.push(task.id);
                            continue 'outer;
                        }
                    }
                    // first appearance of group
                    group_index_map.push((s, grouped.len()));
                    grouped.push(Group { id: Some(s), tasks: vec![task.id] })
                }
                None => grouped.push(Group { id: None, tasks: vec![task.id] }),
            }
        }
        Ok(grouped)
    }
    /// Sort the tasks and return the sorted tasks grouped by their group.
    pub fn sort_grouped_hash_specialization(&mut self) -> Result<Vec<Group<'i, T, G>>, TopologicalError<'i, T, G>>
    where
        G: Eq + Hash,
    {
        let sorted = self.sort()?;
        let mut group_index_map: HashMap<&G, usize> = HashMap::new();
        let mut grouped: Vec<Group<T, G>> = vec![];
        for task in sorted {
            match task.group {
                Some(s) => match group_index_map.get(s) {
                    Some(position) => {
                        grouped[*position].tasks.push(task.id);
                    }
                    None => {
                        let position = grouped.len();
                        group_index_map.insert(s, position);
                        grouped.push(Group { id: Some(s), tasks: vec![task.id] })
                    }
                },
                None => grouped.push(Group { id: None, tasks: vec![task.id] }),
            }
        }
        Ok(grouped)
    }
}

impl<'i, T, G> FinalizeDependencies<'i, T, G>
where
    T: PartialEq,
    G: PartialEq,
{
    fn virtualize(&mut self, task: &Task<'i, T, G>) -> Result<(), TopologicalError<'i, T, G>> {
        let dependent_tasks = self.virtualize_dependent_tasks(&task.dependent_tasks)?;
        let group_id = match task.group {
            Some(reals) => self.virtualize_group(reals)? as isize,
            None => -1,
        };
        self.task_map.push(task.id);
        self.virtualized_groups.push(group_id);
        self.virtualized_dependent_tasks.push(dependent_tasks);
        Ok(())
    }
    fn virtualize_group(&self, group: &'i G) -> Result<usize, TopologicalError<'i, T, G>> {
        match self.group_map.iter().position(|x| *x == group) {
            Some(index) => Ok(index),
            None => Err(TopologicalError::MissingGroup { group })?,
        }
    }
    fn virtualize_dependent_tasks(&self, input: &[&'i T]) -> Result<Vec<usize>, TopologicalError<'i, T, G>> {
        let mut output = Vec::with_capacity(input.len());
        for task in input {
            match self.task_map.iter().position(|x| x == task) {
                Some(index) => output.push(index),
                None => return Err(TopologicalError::MissingTask { task }),
            }
        }
        Ok(output)
    }
}

pub fn double_topological_sort(mut group: Vec<isize>, unique_groups: usize, dependent: Vec<Vec<usize>>) -> Vec<isize> {
    let n = group.len();
    let mut in_degree = vec![0; n];
    let mut adj = vec![vec![]; n];
    for (item, deps) in dependent.iter().enumerate() {
        for &dep in deps {
            adj[dep].push(item);
            in_degree[item] += 1;
        }
    }
    let items_ids = topological_sort(adj, in_degree);
    if items_ids.len() != n {
        return vec![];
    }

    let mut groups = items_ids.into_iter().fold(vec![vec![]; unique_groups], |mut acc, i| {
        if group[i] == -1 {
            group[i] = acc.len() as isize;
            acc.push(vec![]);
        }
        acc[group[i] as usize].push(i as isize);
        acc
    });

    let mut group_in_degree = vec![0; groups.len()];
    let mut group_adj = vec![vec![]; groups.len()];
    for (item, deps) in dependent.into_iter().enumerate() {
        for dep in deps {
            let (src, dst) = (group[dep] as usize, group[item] as usize);
            if src == dst {
                continue;
            }
            group_adj[src].push(dst);
            group_in_degree[dst] += 1;
        }
    }
    let group_ids = topological_sort(group_adj, group_in_degree);
    if group_ids.len() != groups.len() {
        return vec![];
    }

    group_ids.into_iter().fold(Vec::new(), |mut acc, i| {
        acc.append(&mut groups[i]);
        acc
    })
}

fn topological_sort(adj: Vec<Vec<usize>>, mut indeg: Vec<u32>) -> Vec<usize> {
    let mut q = indeg.iter().enumerate().filter(|(_, &d)| d == 0).map(|(node, _)| node).collect::<VecDeque<_>>();
    let mut ret = Vec::new();
    while let Some(node) = q.pop_front() {
        ret.push(node);
        for &new_node in adj[node].iter() {
            indeg[new_node] -= 1;
            if indeg[new_node] == 0 {
                q.push_back(new_node)
            }
        }
    }
    ret
}
