use super::*;

impl<'i, T: Display, G: Display + Ord> DependentSort<'i, T, G> {
    /// Draw a mermaid flowchart of the tasks and their dependencies.
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
