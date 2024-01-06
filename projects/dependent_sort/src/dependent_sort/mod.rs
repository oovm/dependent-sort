use crate::TopologicalError;
use itertools::Itertools;
use std::{
    cmp::Ordering,
    collections::{BTreeMap, VecDeque},
    fmt::{Debug, Display},
    ops::AddAssign,
};

mod arithmetic;
// mod mermaid;

#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct DependentSort<'i, T, G> {
    /// non-virtualized tasks
    tasks: Vec<Task<'i, T, G>>,
    // virtual_group_id: isize,
    /// Should circular dependencies report an error immediately or declare them at the same time.
    allow_circular: bool,
}

impl<'i, T: Display, G: Display + Ord> DependentSort<'i, T, G> {
    pub fn draw_mermaid(&self) -> String {
        let mut out = String::with_capacity(1024);
        out.push_str("flowchart TB\n");
        // define all tasks
        for task in self.tasks.iter() {
            // t0["Task 0"]
            out.push_str(&format!("    t{}[\"Task {}\"]\n", task.id, task.id));
        }
        // define all groups
        let mut groups: BTreeMap<&G, Vec<&Task<T, G>>> = BTreeMap::new();
        for task in &self.tasks {
            if let Some(group) = task.group {
                groups.entry(group).or_default().push(task);
            }
        }
        for (group, tasks) in groups {
            out.push_str(&format!("    subgraph {}\n", group));
            for task in tasks {
                for dep in &task.dependent_tasks {
                    out.push_str(&format!("        t{} --> t{}\n", dep, task.id));
                }
            }
            out.push_str("    end\n");
        }
        // draw lonely tasks
        for task in self.tasks.iter() {
            if task.group.is_none() {
                for dep in &task.dependent_tasks {
                    out.push_str(&format!("    t{} --> t{}\n", dep, task.id));
                }
            }
        }

        out
    }
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

#[derive(Debug)]
struct VirtualSort<'i, T, G> {
    /// maps for recovering the original tasks
    task_map: Vec<&'i T>,
    /// maps for recovering the original groups
    group_map: Vec<&'i G>,
    /// virtualized tasks
    virtualized_groups: Vec<isize>,
    virtualized_dependent_tasks: Vec<Vec<usize>>,
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
    fn finalize(&mut self) -> Result<VirtualSort<'i, T, G>, TopologicalError<'i, T, G>> {
        let mut sorter = VirtualSort::default();
        let tasks = self.tasks.iter().cloned().collect_vec();
        // push all task to task map
        for task in tasks.clone() {
            sorter.task_map.push(task.id);
            if let Some(group) = task.group {
                if !sorter.group_map.contains(&group) {
                    sorter.group_map.push(group);
                }
            }
        }
        for task in tasks {
            sorter.virtualize(task)?;
        }
        Ok(sorter)
    }
    pub fn sort(&mut self) -> Result<Vec<Task<'i, T, G>>, TopologicalError<'i, T, G>> {
        let sorter = self.finalize()?;
        let sorted = double_topological_sort(
            sorter.virtualized_groups.clone(),
            sorter.virtualized_groups.len(),
            sorter.virtualized_dependent_tasks.clone(),
        );
        Ok(sorted.into_iter().map(|i| self.tasks[i as usize].clone()).collect())
    }
}

impl<'i, T, G> VirtualSort<'i, T, G>
where
    T: PartialEq,
    G: PartialEq,
{
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
        for &nnode in adj[node].iter() {
            indeg[nnode] -= 1;
            if indeg[nnode] == 0 {
                q.push_back(nnode)
            }
        }
    }
    ret
}
