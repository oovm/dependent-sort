use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::hash::Hash;
use std::mem::take;
use std::ops::AddAssign;
use itertools::Itertools;
use log::log;
use crate::TopologicalError;

mod arithmetic;

#[derive(Default)]
pub struct DependentSort<'i, T, G> {
    /// non-virtualized tasks
    tasks: Vec<Task<'i, T, G>>,
    // virtualized tasks
    virtualized: Vec<VirtualTasks>,
    // maps for recovering the original tasks
    task_map: Vec<&'i T>,
    /// maps for recovering the original groups
    group_map: Vec<&'i G>,
    virtual_group_id: isize,
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
    // each task is assigned a group id
    // The real tasks are 0, 1, 2, 3...
    // dummy tasks are -1, -2, -3 ...
    group_id: isize,
    dependent_tasks: Vec<usize>,
}

impl<'i, T, G> DependentSort<'i, T, G> where
    T: PartialEq,
    G: PartialEq, {
    pub fn finalize(&mut self) -> Result<(), TopologicalError> where
        T: PartialEq,
        G: PartialEq, {
        let tasks = take(&mut self.tasks);
        for task in tasks {
            self.virtualize(task)?;
        }
        Ok(())
    }
    fn virtualize(&mut self, task: Task<'i, T, G>) -> Result<(), TopologicalError>
        where
            T: PartialEq,
            G: PartialEq,
    {
        let dependent_tasks = self.virtualize_dependent_tasks(&task.dependent_tasks)?;
        match task.group {
            Some(reals) => {
                let group_id = self.virtualize_group(reals)? as isize;
                self.virtualized.push(VirtualTasks {
                    task_id: self.virtualized.len(),
                    group_id,
                    dependent_tasks,
                });
            }
            None => {
                self.virtual_group_id -= 1;
                self.virtualized.push(VirtualTasks {
                    task_id: self.virtualized.len(),
                    group_id: self.virtual_group_id,
                    dependent_tasks,
                });
            }
        }
        Ok(())
    }
    fn virtualize_group(&self, input: &'i G) -> Result<usize, TopologicalError>
        where
            T: PartialEq,
            G: PartialEq,
    {
        match self.group_map.iter().position(|x| *x == input) {
            Some(index) => Ok(index),
            None => Err(TopologicalError::MissingGroup)?,
        }
    }
    fn virtualize_dependent_tasks(&self, input: &[&'i T]) -> Result<Vec<usize>, TopologicalError>
        where
            T: PartialEq,
            G: PartialEq,
    {
        let mut output = Vec::with_capacity(input.len());
        for task in input {
            match self.task_map.iter().position(|x| x == task) {
                Some(index) => output.push(index),
                None => Err(TopologicalError::MissingTask)?,
            }
        }
        Ok(output)
    }
}

impl<'i, T, G> DependentSort<'i, T, G> {
    fn virtual_groups(&self) -> HashMap<usize, VirtualTasks>
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
