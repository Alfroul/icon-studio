use std::collections::VecDeque;

use super::helpers::{element_id, find_element_deep_mut, set_element_gradient_option, set_element_shadow_option, set_element_filter_option};
use super::{Canvas, Element, Gradient, IconProject, Shadow};
use super::filter::SvgFilter;
use crate::engine::text_measure;

pub trait Command: Send {
    fn execute(&mut self, project: &mut IconProject) -> Result<(), String>;
    fn undo(&mut self, project: &mut IconProject) -> Result<(), String>;
    fn as_any(&self) -> &dyn std::any::Any;
    fn label(&self) -> String {
        "Unknown".to_string()
    }
}

/// A batch of commands that undo/redo as a single unit.
struct UndoBatch {
    entries: Vec<Box<dyn Command>>,
    committed: bool,
}

impl Drop for UndoBatch {
    fn drop(&mut self) {
        if !self.committed && !self.entries.is_empty() {
            eprintln!("WARNING: UndoBatch dropped without commit - {} commands lost", self.entries.len());
        }
    }
}

pub struct CommandHistory {
    undo_stack: VecDeque<Box<dyn Command>>,
    redo_stack: VecDeque<Box<dyn Command>>,
    max_size: usize,
    pending_batch: Option<UndoBatch>,
    snapshot_count: usize,
    max_snapshots: usize,
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self {
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            max_size: 200,
            pending_batch: None,
            snapshot_count: 0,
            max_snapshots: 30,
        }
    }
}

impl CommandHistory {
    pub fn new(max_size: usize) -> Self {
        Self {
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            max_size,
            pending_batch: None,
            snapshot_count: 0,
            max_snapshots: 30,
        }
    }

    pub fn push_and_execute(
        &mut self,
        mut cmd: Box<dyn Command>,
        project: &mut IconProject,
    ) -> Result<(), String> {
        cmd.execute(project)?;
        project.bump_version();

        if let Some(ref mut batch) = self.pending_batch {
            batch.entries.push(cmd);
        } else {
            self.redo_stack.clear();
            self.track_snapshot_push(cmd);
        }
        Ok(())
    }

    pub fn undo(&mut self, project: &mut IconProject) -> Result<bool, String> {
        if let Some(mut cmd) = self.undo_stack.pop_back() {
            let is_snap = cmd.as_any().is::<SnapshotCommand>();
            cmd.undo(project)?;
            project.bump_version();
            self.redo_stack.push_back(cmd);
            if is_snap {
                self.snapshot_count = self.snapshot_count.saturating_sub(1);
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn redo(&mut self, project: &mut IconProject) -> Result<bool, String> {
        if let Some(mut cmd) = self.redo_stack.pop_back() {
            let is_snap = cmd.as_any().is::<SnapshotCommand>();
            cmd.execute(project)?;
            project.bump_version();
            self.undo_stack.push_back(cmd);
            if is_snap {
                self.snapshot_count += 1;
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.pending_batch = None;
        self.snapshot_count = 0;
    }

    pub fn history_labels(&self) -> (Vec<String>, Vec<String>) {
        let undo_labels: Vec<String> = self.undo_stack.iter().map(|c| c.label()).collect();
        let redo_labels: Vec<String> = self.redo_stack.iter().rev().map(|c| c.label()).collect();
        (undo_labels, redo_labels)
    }

    pub fn record(&mut self, cmd: Box<dyn Command>) {
        if let Some(ref mut batch) = self.pending_batch {
            batch.entries.push(cmd);
        } else {
            self.redo_stack.clear();
            self.track_snapshot_push(cmd);
        }
    }

    pub fn begin_batch(&mut self, _label: &str) -> Result<(), String> {
        if self.pending_batch.is_some() {
            return Err("Nested batches are not supported".to_string());
        }
        self.pending_batch = Some(UndoBatch { entries: Vec::new(), committed: false });
        Ok(())
    }

    pub fn commit_batch(&mut self) -> Result<(), String> {
        let mut batch = self.pending_batch.take()
            .ok_or("commit_batch called without begin_batch")?;
        batch.committed = true;
        let entries = std::mem::take(&mut batch.entries);
        if !entries.is_empty() {
            self.redo_stack.clear();
            let snap_count = entries.iter().filter(|c| c.as_any().is::<SnapshotCommand>()).count();
            self.undo_stack.push_back(Box::new(BatchCommand { commands: entries }));
            self.snapshot_count += snap_count;
            while self.undo_stack.len() > self.max_size || self.snapshot_count > self.max_snapshots {
                if let Some(removed) = self.undo_stack.pop_front() {
                    if removed.as_any().is::<SnapshotCommand>() {
                        self.snapshot_count = self.snapshot_count.saturating_sub(1);
                    } else if let Some(batch) = removed.as_any().downcast_ref::<BatchCommand>() {
                        self.snapshot_count = self.snapshot_count.saturating_sub(
                            batch.commands.iter().filter(|c| c.as_any().is::<SnapshotCommand>()).count()
                        );
                    }
                } else {
                    break;
                }
            }
        }
        Ok(())
    }

    pub fn run_batch<T>(&mut self, label: &str, f: impl FnOnce(&mut Self) -> T) -> Result<T, String> {
        self.begin_batch(label)?;
        let result = f(self);
        self.commit_batch()?;
        Ok(result)
    }

    fn track_snapshot_push(&mut self, cmd: Box<dyn Command>) {
        let is_snap = cmd.as_any().is::<SnapshotCommand>();
        self.undo_stack.push_back(cmd);
        if is_snap {
            self.snapshot_count += 1;
        }
        while self.undo_stack.len() > self.max_size || self.snapshot_count > self.max_snapshots {
            if let Some(removed) = self.undo_stack.pop_front() {
                if removed.as_any().is::<SnapshotCommand>() {
                    self.snapshot_count = self.snapshot_count.saturating_sub(1);
                }
            } else {
                break;
            }
        }
    }
}

struct BatchCommand {
    commands: Vec<Box<dyn Command>>,
}

impl Command for BatchCommand {
    fn label(&self) -> String { "Batch Operation".to_string() }
    fn execute(&mut self, project: &mut IconProject) -> Result<(), String> {
        let elements_backup = project.active_elements().to_vec();
        let next_id_backup = project.next_element_id;
        let version_backup = project.version;
        for (i, cmd) in self.commands.iter_mut().enumerate() {
            if let Err(e) = cmd.execute(project) {
                for prev_cmd in self.commands[..i].iter_mut().rev() {
                    if let Err(undo_err) = prev_cmd.undo(project) {
                        eprintln!("Warning: undo failed during batch rollback: {}", undo_err);
                    }
                }
                *project.active_elements_mut() = elements_backup;
                project.next_element_id = next_id_backup;
                project.version = version_backup;
                return Err(e);
            }
        }
        Ok(())
    }
    fn undo(&mut self, project: &mut IconProject) -> Result<(), String> {
        for cmd in self.commands.iter_mut().rev() {
            cmd.undo(project)?;
        }
        Ok(())
    }
    fn as_any(&self) -> &dyn std::any::Any { self }
}


pub struct AddElementCommand {
    element: Element,
}

impl AddElementCommand {
    pub fn new(element: Element) -> Self {
        Self { element }
    }
}

impl Command for AddElementCommand {
    fn label(&self) -> String { "Add Element".to_string() }
    fn execute(&mut self, project: &mut IconProject) -> Result<(), String> {
        project.active_elements_mut().push(self.element.clone());
        Ok(())
    }

    fn undo(&mut self, project: &mut IconProject) -> Result<(), String> {
        let id = element_id(&self.element);
        project.active_elements_mut().retain(|e| element_id(e) != id);
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any { self }
}

pub struct RemoveElementCommand {
    element: Element,
    index: usize,
}

impl RemoveElementCommand {
    pub fn new(element: Element, index: usize) -> Self {
        Self { element, index }
    }
}

impl Command for RemoveElementCommand {
    fn label(&self) -> String { "Remove Element".to_string() }
    fn execute(&mut self, project: &mut IconProject) -> Result<(), String> {
        let elements = project.active_elements_mut();
        if self.index < elements.len() {
            elements.remove(self.index);
            Ok(())
        } else {
            Err(format!(
                "RemoveElementCommand: index {} out of bounds (len {})",
                self.index,
                elements.len()
            ))
        }
    }

    fn undo(&mut self, project: &mut IconProject) -> Result<(), String> {
        let elements = project.active_elements_mut();
        let insert_at = self.index.min(elements.len());
        elements.insert(insert_at, self.element.clone());
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any { self }
}

pub struct SetPropsCommand {
    element_id: String,
    old_props: serde_json::Value,
    new_props: serde_json::Value,
}

impl SetPropsCommand {
    pub fn new(
        element_id: String,
        old_props: serde_json::Value,
        new_props: serde_json::Value,
    ) -> Self {
        Self {
            element_id,
            old_props,
            new_props,
        }
    }
}

impl Command for SetPropsCommand {
    fn label(&self) -> String { "Edit Properties".to_string() }
    fn execute(&mut self, project: &mut IconProject) -> Result<(), String> {
        apply_props(project, &self.element_id, &self.new_props)
    }

    fn undo(&mut self, project: &mut IconProject) -> Result<(), String> {
        apply_props(project, &self.element_id, &self.old_props)
    }

    fn as_any(&self) -> &dyn std::any::Any { self }
}

/// Merge-apply: serialize current element → merge incoming props → deserialize back.
fn apply_props(
    project: &mut IconProject,
    target_id: &str,
    props: &serde_json::Value,
) -> Result<(), String> {
    let needs_text_recalc = props.as_object().is_some_and(|obj| {
        obj.keys()
            .any(|k| matches!(k.as_str(), "content" | "font_family" | "font_size" | "font_weight" | "letter_spacing"))
    });

    let elem = find_element_deep_mut(project.active_elements_mut(), target_id)
        .ok_or_else(|| format!("SetPropsCommand: element '{}' not found", target_id))?;

    let mut current = serde_json::to_value(&*elem).map_err(|e| e.to_string())?;
    if let (serde_json::Value::Object(ref mut cur), serde_json::Value::Object(ref incoming)) =
        (&mut current, props)
    {
        for (k, v) in incoming {
            cur.insert(k.clone(), v.clone());
        }
    }
    *elem = serde_json::from_value(current).map_err(|e| e.to_string())?;

    if needs_text_recalc {
        if let Element::Text(ref mut t) = elem {
            let weight_u16 = match t.font_weight.as_str() {
                "light" => 300,
                "normal" => 400,
                "medium" => 500,
                "semibold" => 600,
                "bold" => 700,
                other => other.parse::<u16>().unwrap_or(400),
            };
            let measured = text_measure::measure_text_width(
                &t.content, &t.font_family, t.font_size as f32, weight_u16,
            ) as f64;
            // Ensure minimum positive width so subsequent set_props validations don't fail
            t.common.width = if measured > 0.0 { measured } else { t.font_size * 0.5 };
            t.common.height = t.font_size * 1.2;
        }
    }

    let needs_path_recalc = props.as_object().is_some_and(|obj| obj.contains_key("d"));
    if needs_path_recalc {
        crate::model::helpers::recompute_path_natural_dims(elem);
    }

    Ok(())
}

pub struct ReorderCommand {
    old_index: usize,
    new_index: usize,
}

impl ReorderCommand {
    pub fn new(_element_id: String, old_index: usize, new_index: usize) -> Self {
        Self {
            old_index,
            new_index,
        }
    }
}

impl Command for ReorderCommand {
    fn label(&self) -> String { "Reorder".to_string() }
    fn execute(&mut self, project: &mut IconProject) -> Result<(), String> {
        move_element(project, self.old_index, self.new_index)
    }

    fn undo(&mut self, project: &mut IconProject) -> Result<(), String> {
        move_element(project, self.new_index, self.old_index)
    }

    fn as_any(&self) -> &dyn std::any::Any { self }
}

fn move_element(
    project: &mut IconProject,
    from: usize,
    to: usize,
) -> Result<(), String> {
    let elements = project.active_elements_mut();
    if from >= elements.len() || to >= elements.len() {
        return Err(format!(
            "ReorderCommand: index out of bounds (from={}, to={}, len={})",
            from,
            to,
            elements.len()
        ));
    }
    let elem = elements.remove(from);
    elements.insert(to, elem);
    Ok(())
}

pub struct SnapshotCommand {
    before: Vec<Element>,
    after: Vec<Element>,
    before_next_id: u64,
    after_next_id: u64,
}

impl SnapshotCommand {
    pub fn new(before: Vec<Element>, before_next_id: u64, after: Vec<Element>, after_next_id: u64) -> Self {
        Self { before, after, before_next_id, after_next_id }
    }
}

impl Command for SnapshotCommand {
    fn label(&self) -> String { "Snapshot".to_string() }
    fn execute(&mut self, project: &mut IconProject) -> Result<(), String> {
        *project.active_elements_mut() = self.after.clone();
        project.next_element_id = self.after_next_id;
        Ok(())
    }

    fn undo(&mut self, project: &mut IconProject) -> Result<(), String> {
        *project.active_elements_mut() = self.before.clone();
        project.next_element_id = self.before_next_id;
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any { self }
}

pub struct SetGradientCommand {
    element_id: String,
    old_gradient: Option<Gradient>,
    new_gradient: Option<Gradient>,
}

impl SetGradientCommand {
    pub fn new(element_id: String, old_gradient: Option<Gradient>, new_gradient: Option<Gradient>) -> Self {
        Self { element_id, old_gradient, new_gradient }
    }
}

impl Command for SetGradientCommand {
    fn label(&self) -> String { "Change Gradient".to_string() }
    fn execute(&mut self, project: &mut IconProject) -> Result<(), String> {
        let elem = find_element_deep_mut(project.active_elements_mut(), &self.element_id)
            .ok_or_else(|| format!("Element {} not found", self.element_id))?;
        set_element_gradient_option(elem, self.new_gradient.clone());
        Ok(())
    }

    fn undo(&mut self, project: &mut IconProject) -> Result<(), String> {
        let elem = find_element_deep_mut(project.active_elements_mut(), &self.element_id)
            .ok_or_else(|| format!("Element {} not found", self.element_id))?;
        set_element_gradient_option(elem, self.old_gradient.clone());
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any { self }
}

pub struct SetShadowCommand {
    element_id: String,
    old_shadow: Option<Shadow>,
    new_shadow: Option<Shadow>,
}

impl SetShadowCommand {
    pub fn new(element_id: String, old_shadow: Option<Shadow>, new_shadow: Option<Shadow>) -> Self {
        Self { element_id, old_shadow, new_shadow }
    }
}

impl Command for SetShadowCommand {
    fn label(&self) -> String { "Change Shadow".to_string() }
    fn execute(&mut self, project: &mut IconProject) -> Result<(), String> {
        let elem = find_element_deep_mut(project.active_elements_mut(), &self.element_id)
            .ok_or_else(|| format!("Element {} not found", self.element_id))?;
        set_element_shadow_option(elem, self.new_shadow.clone());
        Ok(())
    }

    fn undo(&mut self, project: &mut IconProject) -> Result<(), String> {
        let elem = find_element_deep_mut(project.active_elements_mut(), &self.element_id)
            .ok_or_else(|| format!("Element {} not found", self.element_id))?;
        set_element_shadow_option(elem, self.old_shadow.clone());
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any { self }
}

pub struct SetFilterCommand {
    element_id: String,
    old_filter: Option<SvgFilter>,
    new_filter: Option<SvgFilter>,
}

impl SetFilterCommand {
    pub fn new(element_id: String, old_filter: Option<SvgFilter>, new_filter: Option<SvgFilter>) -> Self {
        Self { element_id, old_filter, new_filter }
    }
}

impl Command for SetFilterCommand {
    fn label(&self) -> String { "Change Filter".to_string() }
    fn execute(&mut self, project: &mut IconProject) -> Result<(), String> {
        let elem = find_element_deep_mut(project.active_elements_mut(), &self.element_id)
            .ok_or_else(|| format!("Element {} not found", self.element_id))?;
        set_element_filter_option(elem, self.new_filter.clone());
        Ok(())
    }

    fn undo(&mut self, project: &mut IconProject) -> Result<(), String> {
        let elem = find_element_deep_mut(project.active_elements_mut(), &self.element_id)
            .ok_or_else(|| format!("Element {} not found", self.element_id))?;
        set_element_filter_option(elem, self.old_filter.clone());
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any { self }
}

pub struct CanvasCommand {
    old_canvas: Canvas,
    new_canvas: Canvas,
}

impl CanvasCommand {
    pub fn new(old: Canvas, new: Canvas) -> Self {
        Self { old_canvas: old, new_canvas: new }
    }
}

impl Command for CanvasCommand {
    fn label(&self) -> String { "Edit Canvas".to_string() }
    fn execute(&mut self, project: &mut IconProject) -> Result<(), String> {
        *project.active_canvas_mut() = self.new_canvas.clone();
        Ok(())
    }
    fn undo(&mut self, project: &mut IconProject) -> Result<(), String> {
        *project.active_canvas_mut() = self.old_canvas.clone();
        Ok(())
    }
    fn as_any(&self) -> &dyn std::any::Any { self }
}
