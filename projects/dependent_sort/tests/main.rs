use dependent_sort::{DependentSort, Task};

#[test]
fn ready() {
    println!("it works!")
}

#[test]
fn execution_order() {
    let mut tasks = DependentSort::default();
    tasks += Task::new(&0);
    tasks += Task::new_with_dependent(&1, vec![&6]);
    tasks += Task::new_with_dependent(&2, vec![&5]).with_group(&"A");
    tasks += Task::new_with_dependent(&3, vec![&6]).with_group(&"B");
    tasks += Task::new_with_dependent(&4, vec![&2, &3, &6]).with_group(&"B");
    tasks += Task::new_with_dependent(&5, vec![]).with_group(&"A");
    tasks += Task::new_with_dependent(&6, vec![]).with_group(&"B");
    tasks += Task::new(&7);
    println!("{}", tasks.draw_mermaid());
    let sorted = tasks.sort().unwrap();
    for task in &sorted {
        println!("{:?}", task);
    }
    let ids = sorted.into_iter().map(|task| *task.id).collect::<Vec<_>>();
    assert_eq!(ids, vec![5, 2, 0, 7, 6, 3, 4, 1])
}
