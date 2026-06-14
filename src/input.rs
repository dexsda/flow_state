use std::io::Result;

use ratatui::{
    backend::Backend,
    crossterm::event::{self, Event, KeyCode},
    Terminal,
};

use crate::{
    app::{App, CurrentScreen, ScreenMode}, habit::{HabitStatus, HabitType, Day}, list::ChecklistType, ui::ui
};

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Release {
                continue;
            }

            if handle_global_keys(key.code, app) {
                break;
            }

            match app.current_screen {
                CurrentScreen::Today => handle_today_keys(key.code, app),
                CurrentScreen::Stats => handle_stats_keys(key.code, app),
                CurrentScreen::Heatmap => handle_heatmap_keys(key.code, app),
            }
        }
    }
    Ok(())
}

fn handle_global_keys(code: KeyCode, app: &mut App) -> bool {
    match code {
        KeyCode::Char('q') => {
            if let ScreenMode::Normal = app.screen_mode {
                let _ = app.save_habits();
                return true;
            }
        }
        KeyCode::Esc => {
            if !matches!(app.screen_mode, ScreenMode::Normal) {
                app.toggle_normal_mode();
            } else {
                let _ = app.save_habits();
                return true;
            }
        }
        _ => {}
    }
    false
}

fn handle_today_keys(code: KeyCode, app: &mut App) {
    match app.screen_mode {
        ScreenMode::Normal => handle_normal_mode(code, app),
        ScreenMode::Adding => handle_text_input(code, app, false),
        ScreenMode::Editing => handle_text_input(code, app, true),
        ScreenMode::Deleting => handle_confirm(code, app, true),
        ScreenMode::Reset => handle_confirm(code, app, false),
        ScreenMode::List => handle_list_edit_mode(code, app),
        ScreenMode::ListAdd => handle_list_edit_add_mode(code, app),
        ScreenMode::CheckOff => handle_checkoff(code, app),
    }
}

fn handle_stats_keys(code: KeyCode, app: &mut App) {
    if let KeyCode::Tab = code {
        app.toggle_page();
    }
}

fn handle_heatmap_keys(code: KeyCode, app: &mut App) {
    match code {
        KeyCode::Tab => app.toggle_page(),
        KeyCode::Char('j') | KeyCode::Down => app.increment_year_counter(),
        KeyCode::Char('k') | KeyCode::Up => app.decrement_year_counter(),
        _ => {}
    }
}

fn handle_normal_mode(code: KeyCode, app: &mut App) {
    match code {
        KeyCode::Tab => app.toggle_page(),
        KeyCode::Char('j') | KeyCode::Down => app.increment_habits_counter(),
        KeyCode::Char('k') | KeyCode::Up => app.decrement_habits_counter(),
        KeyCode::Char('h') | KeyCode::Left => app.toggle_build_habits(),
        KeyCode::Char('l') | KeyCode::Right => app.toggle_avoid_habit(),
        KeyCode::Char('a') => app.toggle_add_mode(),
        KeyCode::Char('A') => {
            let habit = app.get_selected_habit();
            if habit.checklist.checklist_type != ChecklistType::None {
                app.toggle_edit_list(habit);
            }
        }
        KeyCode::Char('e') => {
            let habit = app.get_selected_habit();
            app.toggle_edit_mode(habit);
        }
        KeyCode::Char('d') => app.toggle_delete_mode(),
        KeyCode::Char('r') => app.toggle_reset_mode(),
        KeyCode::Char('y') => app.toggle_day(),
        KeyCode::Char('c') => app.toggle_checklist_type(),
        KeyCode::Enter | KeyCode::Char(' ') => {
            let habit = app.get_selected_habit();
            if habit.checklist.checklist_type == ChecklistType::Todo 
                && !habit.checklist.checklist.is_empty() 
                && habit.check_status(&Day::Today) == HabitStatus::InComplete {
                app.toggle_checkoff_mode(habit);
            } else {
                app.toggle_current_habit();
                let _ = app.save_habits();
            }
        }
        _ => {}
    }
}

fn handle_checkoff(code: KeyCode, app: &mut App) {
    match code {
        KeyCode::Char('j') | KeyCode::Down => app.increment_list_counter(),
        KeyCode::Char('k') | KeyCode::Up => app.decrement_list_counter(),
        KeyCode::Char(' ') | KeyCode::Enter => {
            app.delete_list_item();
            match app.current_habit.habit_type {
                HabitType::Build => {
                    app.build_habits[app.counter.build_counter] = app.current_habit.clone();
                }
                HabitType::Avoid => {
                    app.avoid_habits[app.counter.avoid_counter] = app.current_habit.clone();
                }
            }
            app.toggle_current_habit();
            app.toggle_normal_mode();
        }
        _ => {}
    }
}

fn handle_list_edit_mode(code: KeyCode, app: &mut App) {
    match code {
        KeyCode::Char('j') | KeyCode::Down => app.increment_list_counter(),
        KeyCode::Char('k') | KeyCode::Up => app.decrement_list_counter(),
        KeyCode::Char('a') => app.add_list_item(),
        KeyCode::Char('e') => app.edit_list_item(),
        KeyCode::Char('d') | KeyCode::Delete => {
            app.delete_list_item();
            match app.current_habit.habit_type {
                HabitType::Build => {
                    app.build_habits[app.counter.build_counter] = app.current_habit.clone();
                }
                HabitType::Avoid => {
                    app.avoid_habits[app.counter.avoid_counter] = app.current_habit.clone();
                }
            }
        }
        _ => {}
    }
}

fn handle_list_edit_add_mode(code: KeyCode, app: &mut App) {
    match code {
        KeyCode::Backspace => {
            app.current_habit.checklist.checklist[app.list_index].pop();
        }
        KeyCode::Enter => {
            match app.current_habit.habit_type {
                HabitType::Build => {
                    app.build_habits[app.counter.build_counter] = app.current_habit.clone();
                }
                HabitType::Avoid => {
                    app.avoid_habits[app.counter.avoid_counter] = app.current_habit.clone();
                }
            }
            app.screen_mode = ScreenMode::List;
        }
        KeyCode::Char(value) => {
            app.current_habit.checklist.checklist[app.list_index].push(value);
        }
        _ => {}
    }
}

fn handle_text_input(code: KeyCode, app: &mut App, is_editing: bool) {
    match code {
        KeyCode::Tab => app.toggle_habit_type(),
        KeyCode::Backspace => {
            app.current_habit.name.pop();
        }
        KeyCode::Enter => {
            if is_editing {
                app.edit_habit();
            } else {
                app.add_habit();
            }
        }
        KeyCode::Char(value) => {
            app.current_habit.name.push(value);
        }
        _ => {}
    }
}

fn handle_confirm(code: KeyCode, app: &mut App, is_delete: bool) {
    match code {
        KeyCode::Char('y') => {
            if is_delete {
                app.delete_current_habit();
            } else {
                app.reset_current_habit();
            }
        }
        KeyCode::Char('n') => app.toggle_normal_mode(),
        _ => {}
    }
}
