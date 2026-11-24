//! Undo and redo: snapshots of the whole project (it is small). Changes
//! that follow each other quickly, like typing a name, form one step.

use model::Project;

/// Changes within this many seconds of a step's first change are undone
/// together.
const MERGE_SECS: f64 = 1.0;
/// Most undo steps kept.
const MAX_STEPS: usize = 100;

#[derive(Default)]
pub struct History {
    undo: Vec<Project>,
    redo: Vec<Project>,
    /// The project as last seen.
    current: Project,
    /// When the newest undo step began.
    step_started: f64,
}

impl History {
    pub fn new(project: &Project) -> Self {
        Self { current: project.clone(), step_started: f64::NEG_INFINITY, ..Default::default() }
    }

    /// Notes the project after a frame at time `now` (seconds).
    pub fn record(&mut self, project: &Project, now: f64) {
        if *project == self.current {
            return;
        }
        let previous = std::mem::replace(&mut self.current, project.clone());
        if self.undo.is_empty() || now - self.step_started > MERGE_SECS {
            self.undo.push(previous);
            if self.undo.len() > MAX_STEPS {
                self.undo.remove(0);
            }
            self.step_started = now;
        }
        self.redo.clear();
    }

    /// Takes `project` as the current state without making it a step.
    pub fn sync(&mut self, project: &Project) {
        self.current = project.clone();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Restores the step before. Returns whether there was one.
    pub fn undo(&mut self, project: &mut Project) -> bool {
        self.step_started = f64::NEG_INFINITY; // the next change is a new step
        Self::step(&mut self.undo, &mut self.redo, &mut self.current, project)
    }

    pub fn redo(&mut self, project: &mut Project) -> bool {
        self.step_started = f64::NEG_INFINITY;
        Self::step(&mut self.redo, &mut self.undo, &mut self.current, project)
    }

    fn step(
        from: &mut Vec<Project>,
        to: &mut Vec<Project>,
        current: &mut Project,
        project: &mut Project,
    ) -> bool {
        let Some(target) = from.pop() else { return false };
        to.push(std::mem::replace(current, target.clone()));
        *project = target;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn titled(t: &str) -> Project {
        let mut p = Project::default();
        p.meta.title = t.into();
        p
    }

    #[test]
    fn undo_and_redo_restore_steps() {
        let mut h = History::new(&titled("A"));
        h.record(&titled("B"), 10.0);
        h.record(&titled("C"), 20.0);
        let mut p = titled("C");
        assert!(h.undo(&mut p));
        assert_eq!(p.meta.title, "B");
        assert!(h.undo(&mut p));
        assert_eq!(p.meta.title, "A");
        assert!(!h.undo(&mut p));
        assert!(h.redo(&mut p));
        assert_eq!(p.meta.title, "B");
    }

    #[test]
    fn quick_changes_are_one_step() {
        let mut h = History::new(&titled(""));
        for (i, t) in ["V", "Ve", "Ver"].iter().enumerate() {
            h.record(&titled(t), 10.0 + i as f64 * 0.2);
        }
        let mut p = titled("Ver");
        assert!(h.undo(&mut p));
        assert_eq!(p.meta.title, "");
        assert!(!h.can_undo());
    }

    #[test]
    fn steady_input_still_makes_several_steps() {
        let mut h = History::new(&titled(""));
        for i in 0..10 {
            h.record(&titled(&i.to_string()), i as f64 * 0.5);
        }
        assert!(h.undo.len() > 1);
    }

    #[test]
    fn a_change_right_after_undo_is_its_own_step() {
        let mut h = History::new(&titled("A"));
        h.record(&titled("B"), 10.0);
        let mut p = titled("B");
        h.undo(&mut p);
        h.record(&titled("C"), 10.2);
        h.undo(&mut p);
        assert_eq!(p.meta.title, "A");
    }

    #[test]
    fn a_new_change_clears_redo_and_steps_are_capped() {
        let mut h = History::new(&titled("0"));
        for i in 1..=MAX_STEPS + 5 {
            h.record(&titled(&i.to_string()), i as f64 * 10.0);
        }
        assert_eq!(h.undo.len(), MAX_STEPS);
        let mut p = h.current.clone();
        h.undo(&mut p);
        h.record(&titled("x"), 1e6);
        assert!(!h.can_redo());
    }
}
