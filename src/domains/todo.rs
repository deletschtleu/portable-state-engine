//! Reference domain A — Todo list (`SPEC.md` Section 5, `FR-026`).

use std::collections::BTreeSet;

use crate::core::contracts::{AggregateDefinition, DomainEvent, ExplicitContext};
use crate::core::errors::DomainError;

/// A single Todo item within a Todo list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodoItem {
    pub id: String,
    pub title: String,
    pub completed: bool,
}

/// Immutable state of a Todo list aggregate (`SPEC.md` Section 5).
///
/// Active `todos` are stored in deterministic append order, and `used_ids` retains
/// all IDs ever added so deleted IDs cannot be reused after replay.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TodoListState {
    pub todos: Vec<TodoItem>,
    pub used_ids: BTreeSet<String>,
}

impl TodoListState {
    pub fn get(&self, id: &str) -> Option<&TodoItem> {
        self.todos.iter().find(|item| item.id == id)
    }

    pub fn items(&self) -> &[TodoItem] {
        &self.todos
    }
}

/// Commands accepted by the Todo list domain (`SPEC.md` Section 5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TodoCommand {
    AddTodo { id: String, title: String },
    CompleteTodo { id: String },
    DeleteTodo { id: String },
}

/// Events emitted by the Todo list domain (`SPEC.md` Section 5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TodoEvent {
    TodoAdded { id: String, title: String },
    TodoCompleted { id: String },
    TodoDeleted { id: String },
}

impl DomainEvent for TodoEvent {
    fn event_type(&self) -> &'static str {
        match self {
            TodoEvent::TodoAdded { .. } => "TodoAdded",
            TodoEvent::TodoCompleted { .. } => "TodoCompleted",
            TodoEvent::TodoDeleted { .. } => "TodoDeleted",
        }
    }
}

/// Pure domain definition for the Todo list aggregate.
#[derive(Debug, Clone, Copy, Default)]
pub struct TodoDomain;

impl AggregateDefinition for TodoDomain {
    type State = TodoListState;
    type Command = TodoCommand;
    type Event = TodoEvent;

    fn initial_state(&self) -> Self::State {
        TodoListState::default()
    }

    fn decide(
        &self,
        state: &Self::State,
        command: &Self::Command,
        _context: &ExplicitContext,
    ) -> Result<Vec<Self::Event>, DomainError> {
        match command {
            TodoCommand::AddTodo { id, title } => {
                if id.trim().is_empty() {
                    return Err(DomainError::new(
                        "INVALID_TODO_ID",
                        "todo id must not be blank",
                    ));
                }
                if title.trim().is_empty() {
                    return Err(DomainError::new(
                        "BLANK_TODO_TITLE",
                        "todo title must not be blank or whitespace-only",
                    ));
                }
                if state.used_ids.contains(id) {
                    return Err(DomainError::new(
                        "DUPLICATE_TODO_ID",
                        format!("todo id '{}' has already been used in this list", id),
                    ));
                }
                Ok(vec![TodoEvent::TodoAdded {
                    id: id.clone(),
                    title: title.clone(),
                }])
            }
            TodoCommand::CompleteTodo { id } => {
                let Some(item) = state.get(id) else {
                    return Err(DomainError::new(
                        "TODO_NOT_FOUND",
                        format!("todo id '{}' does not exist", id),
                    ));
                };
                if item.completed {
                    return Ok(Vec::new());
                }
                Ok(vec![TodoEvent::TodoCompleted { id: id.clone() }])
            }
            TodoCommand::DeleteTodo { id } => {
                if state.get(id).is_none() {
                    return Err(DomainError::new(
                        "TODO_NOT_FOUND",
                        format!("todo id '{}' does not exist", id),
                    ));
                }
                Ok(vec![TodoEvent::TodoDeleted { id: id.clone() }])
            }
        }
    }

    fn evolve(&self, state: &Self::State, event: &Self::Event) -> Self::State {
        match event {
            TodoEvent::TodoAdded { id, title } => {
                let mut todos = state.todos.clone();
                todos.push(TodoItem {
                    id: id.clone(),
                    title: title.clone(),
                    completed: false,
                });
                let mut used_ids = state.used_ids.clone();
                used_ids.insert(id.clone());
                TodoListState { todos, used_ids }
            }
            TodoEvent::TodoCompleted { id } => {
                let todos = state
                    .todos
                    .iter()
                    .map(|item| {
                        if item.id == *id {
                            TodoItem {
                                id: item.id.clone(),
                                title: item.title.clone(),
                                completed: true,
                            }
                        } else {
                            item.clone()
                        }
                    })
                    .collect();
                TodoListState {
                    todos,
                    used_ids: state.used_ids.clone(),
                }
            }
            TodoEvent::TodoDeleted { id } => {
                let todos = state
                    .todos
                    .iter()
                    .filter(|item| item.id != *id)
                    .cloned()
                    .collect();
                TodoListState {
                    todos,
                    used_ids: state.used_ids.clone(),
                }
            }
        }
    }
}
