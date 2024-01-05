use crate::TopologicalError;
use itertools::Itertools;
use std::{cmp::Ordering, collections::VecDeque, fmt::Debug, ops::AddAssign};

mod arithmetic;

#[derive(Default)]
pub struct DependentSort<'i, T, G> {
    /// non-virtualized tasks
    tasks: Vec<Task<'i, T, G>>,
    /// maps for recovering the original tasks
    task_map: Vec<&'i T>,
    /// maps for recovering the original groups
    group_map: Vec<&'i G>,
    /// virtualized tasks
    virtualized_groups: Vec<isize>,
    virtualized_dependent_tasks: Vec<Vec<usize>>,
    /// virtual_group_id: isize,
    /// Should circular dependencies report an error immediately or declare them at the same time?
    allow_circular: bool,
}

#[derive(Debug)]
pub struct Task<'i, T, G> {
    pub id: &'i T,
    pub group: Option<&'i G>,
    pub dependent_tasks: Vec<&'i T>,
}

#[derive(Debug)]
struct VirtualTasks {
    task_id: usize,
    group_id: isize,
    dependent_tasks: Vec<usize>,
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

impl<'i, T, G> DependentSort<'i, T, G>
where
    T: PartialEq,
    G: PartialEq,
{
    pub fn finalize(&mut self) -> Result<(), TopologicalError<'i, T, G>> {
        let tasks = self.tasks.iter().cloned().collect_vec();
        // push all task to task map
        for task in tasks.clone() {
            self.task_map.push(task.id);
        }
        // push all group to task map
        for task in tasks.clone() {
            if let Some(group) = task.group {
                if !self.group_map.contains(&group) {
                    self.group_map.push(group);
                }
            }
        }
        for task in tasks {
            self.virtualize(task)?;
        }
        Ok(())
    }
    fn virtualize(&mut self, task: Task<'i, T, G>) -> Result<(), TopologicalError<'i, T, G>> {
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

impl<'i, T, G> DependentSort<'i, T, G>
where
    T: PartialEq,
    G: PartialEq,
{
    pub fn sort(&mut self) -> Result<Vec<isize>, TopologicalError<'i, T, G>> {
        self.finalize()?;
        let sorted = double_topological_sort(
            self.virtualized_groups.clone(),
            self.virtualized_groups.len(),
            self.virtualized_dependent_tasks.clone(),
        );
        Ok(sorted)
    }
}

pub fn double_topological_sort(mut group: Vec<isize>, unique_groups: usize, dependent: Vec<Vec<usize>>) -> Vec<isize> {
    println!("{:?}", group);

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
        for &nnode in adj[node].iter() {
            indeg[nnode] -= 1;
            if indeg[nnode] == 0 {
                q.push_back(nnode)
            }
        }
    }
    ret
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

    // let groups = tasks.virtual_groups();
    // println!("{:#?}", groups);

    let sorted = tasks.sort().unwrap();
    assert_eq!(sorted, vec![5, 2, 6, 3, 4, 0, 7, 1]);
}
