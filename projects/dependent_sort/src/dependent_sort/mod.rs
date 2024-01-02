use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;
use std::ops::AddAssign;

#[derive(Default)]
pub struct DependentSort<T, G> {
    tasks: BTreeMap<T, Task<T, G>>,
}

#[derive(Debug)]
struct VirtualGroup<T> {
    id: usize,
    dependent_groups: Vec<usize>,
    contain_tasks: Vec<T>,
}

pub struct Task<T, G> {
    pub id: T,
    pub group: Option<G>,
    pub dependent_tasks: Vec<T>,
}

impl<T, G> AddAssign<Task<T, G>> for DependentSort<T, G>
    where
        T: Clone + Eq + Hash,
        G: Clone + Eq + Hash,
{
    fn add_assign(&mut self, task: Task<T, G>) {
        self.tasks.insert(task.id.clone(), task);
    }
}

impl<T, G> DependentSort<T, G> {
    /// - First assign a group number to each grouped task
    /// - Assign auto-increment ids to tasks without groups
    fn virtual_groups(&self) -> BTreeMap<usize, VirtualGroup<T>>
        where
            T: Clone,
            G: Eq + Hash,
    {
        let count_groups = self.tasks.values().filter(|s| s.group.is_some()).count();
        let mut real_id = 0;
        let mut virtual_id = count_groups + 1;
        let mut groups = BTreeMap::new();
        let mut group_map = HashMap::new();
        for task in self.tasks.values() {
            match task.group {
                Some(ref group) => {
                    let id = group_map.entry(group).or_insert_with(|| {
                        let id = real_id;
                        real_id += 1;
                        id
                    });
                    let group = groups.entry(*id).or_insert_with(|| VirtualGroup {
                        id: *id,
                        dependent_groups: vec![],
                        contain_tasks: vec![],
                    });
                    group.contain_tasks.push(task.id.clone());
                }
                None => {
                    let id = virtual_id;
                    virtual_id += 1;
                    let group = groups.entry(id).or_insert_with(|| VirtualGroup {
                        id,
                        dependent_groups: vec![],
                        contain_tasks: vec![],
                    });
                    group.contain_tasks.push(task.id.clone());
                }
            }
        }
        groups
    }
    // 1.定义虚拟组
    //   - 先给每一个有组的任务分配一个组编号
    //   - 再给没有组的任务分配自增 id
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

#[test]
fn test() {
    let mut tasks = DependentSort::default();
    tasks += Task { id: 0, group: None, dependent_tasks: vec![] };
    tasks += Task { id: 1, group: None, dependent_tasks: vec![6] };
    tasks += Task { id: 2, group: Some("A"), dependent_tasks: vec![5] };
    tasks += Task { id: 3, group: Some("B"), dependent_tasks: vec![6] };
    tasks += Task { id: 4, group: Some("B"), dependent_tasks: vec![3, 6] };
    tasks += Task { id: 5, group: Some("A"), dependent_tasks: vec![] };
    tasks += Task { id: 6, group: Some("B"), dependent_tasks: vec![] };
    tasks += Task { id: 7, group: None, dependent_tasks: vec![] };

    let groups = tasks.virtual_groups();
    println!("{:#?}", groups);

    let sorted = tasks.sort();
    assert_eq!(sorted, vec![6, 3, 4, 1, 5, 2, 0, 7]);
}
