//! Local visibility and finding-specific feedback for Engram memory.

use crate::engram_planner::EngramPlan;
use crate::sonic_intelligence::{KnowledgeEntry, MemoryCategory, OutcomeSignal, SonicIntelligence};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeliveryStatus {
    Included,
    Trimmed,
    Disabled,
    NoSelection,
    BudgetedOut,
}

#[derive(Debug, Default)]
pub struct MemoryReview {
    open: bool,
    cursor: usize,
    last_plan: Option<EngramPlan>,
    plan_enabled: bool,
    delivery: Option<DeliveryStatus>,
}

impl MemoryReview {
    pub fn open(&mut self) {
        self.open = true;
        self.cursor = 0;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn record_plan(&mut self, plan: EngramPlan, enabled: bool) {
        self.last_plan = Some(plan);
        self.plan_enabled = enabled;
        self.delivery = None;
    }

    pub fn record_delivery(
        &mut self,
        enabled: bool,
        selected_count: usize,
        relevant_count: usize,
        trimmed: bool,
    ) {
        self.delivery = Some(if !enabled {
            DeliveryStatus::Disabled
        } else if selected_count == 0 {
            if relevant_count > 0 {
                DeliveryStatus::BudgetedOut
            } else {
                DeliveryStatus::NoSelection
            }
        } else if trimmed {
            DeliveryStatus::Trimmed
        } else {
            DeliveryStatus::Included
        });
    }

    pub fn move_cursor(
        &mut self,
        direction: isize,
        intelligence: &SonicIntelligence,
        experiment_id: Option<&str>,
    ) {
        let count = experiment_id
            .map(|id| rateable_findings(intelligence, id).len())
            .unwrap_or_default();
        if count == 0 {
            self.cursor = 0;
            return;
        }
        let last = count - 1;
        self.cursor = if direction < 0 {
            self.cursor.saturating_sub(1)
        } else {
            self.cursor.saturating_add(1).min(last)
        };
    }

    pub fn selected_finding_id(
        &self,
        intelligence: &SonicIntelligence,
        experiment_id: Option<&str>,
    ) -> Option<String> {
        let id = experiment_id?;
        let findings = rateable_findings(intelligence, id);
        findings
            .get(self.cursor.min(findings.len().saturating_sub(1)))
            .map(|entry| entry.id.clone())
    }
}

fn rateable_findings<'a>(
    intelligence: &'a SonicIntelligence,
    experiment_id: &str,
) -> Vec<&'a KnowledgeEntry> {
    intelligence
        .knowledge_entries_for_experiment(experiment_id)
        .into_iter()
        .filter(|entry| {
            !entry.heuristic_only
                && matches!(
                    entry.category,
                    MemoryCategory::Musical | MemoryCategory::Experiential
                )
                && !is_descriptive_statistic(entry)
        })
        .collect()
}

fn is_descriptive_statistic(entry: &KnowledgeEntry) -> bool {
    [
        "section_count",
        "slider_count",
        "cycle_count",
        "visualizers",
    ]
    .iter()
    .any(|suffix| entry.source_finding.ends_with(suffix))
}

pub fn render_overlay(
    frame: &mut Frame,
    review: &MemoryReview,
    intelligence: &SonicIntelligence,
    active_experiment_id: Option<&str>,
    advisory_enabled: bool,
) {
    if !review.open {
        return;
    }

    let area = centered_rect(90, 86, frame.area());
    frame.render_widget(Clear, area);

    let mut lines = vec![Line::from(vec![
        Span::styled("Advisory injection: ", Style::default().fg(Color::Gray)),
        Span::styled(
            if advisory_enabled { "ON" } else { "OFF" },
            if advisory_enabled {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        ),
        Span::raw("  [Ctrl+A toggles]"),
    ])];

    if let Some(plan) = &review.last_plan {
        lines.push(Line::from(format!(
            "Most recent plan: model {} | budget {} est. tokens | {} eligible, {} relevant, {} selected",
            display_atom(&plan.model_name, 64),
            plan.token_budget,
            plan.eligible_candidates,
            plan.relevant_candidates,
            plan.selected.len()
        )));
        let delivery = match review.delivery {
            Some(DeliveryStatus::Included) => "included in the compiled request",
            Some(DeliveryStatus::Trimmed) => "selected, then trimmed to fit the request budget",
            Some(DeliveryStatus::Disabled) => "preview only; injection was OFF for that request",
            Some(DeliveryStatus::NoSelection) => "no eligible relevant note was selected",
            Some(DeliveryStatus::BudgetedOut) => {
                "relevant notes existed but exceeded the Engram memory budget"
            }
            None if !review.plan_enabled => "preview only; injection was OFF for that request",
            None => "waiting for prompt compiler budget result",
        };
        lines.push(Line::from(format!("Prompt result: {delivery}")));
        lines.push(Line::from("Selected notes:"));
        if plan.selected.is_empty() {
            let explanation = if plan.relevant_candidates == 0 {
                "  No evidence-backed notes matched the last request."
            } else {
                "  Relevant evidence existed but did not fit the Engram memory budget."
            };
            lines.push(Line::from(explanation));
        } else {
            for planned in &plan.selected {
                let statement = intelligence
                    .store
                    .knowledge
                    .iter()
                    .find(|entry| entry.id == planned.id)
                    .map(|entry| display_atom(&entry.statement, 150))
                    .unwrap_or_else(|| "Stored finding is no longer available.".to_string());
                lines.push(Line::from(format!(
                    "  [{}] score {:.2}, {} term hits, ~{} tokens: {}",
                    category_label(planned.category),
                    planned.score,
                    planned.relevance_hits,
                    planned.estimated_tokens,
                    statement
                )));
            }
        }
    } else {
        lines.push(Line::from(
            "No memory plan has been generated in this session yet.",
        ));
    }

    lines.push(Line::from(""));
    match active_experiment_id {
        Some(experiment_id) => {
            let findings = rateable_findings(intelligence, experiment_id);
            lines.push(Line::from(format!(
                "Finding-specific feedback for active experiment {}:",
                display_atom(experiment_id, 70)
            )));
            if findings.is_empty() {
                lines.push(Line::from(
                    "  No rateable musical/experiential findings are linked to this composition.",
                ));
            } else {
                let selected = *findings
                    .get(review.cursor.min(findings.len() - 1))
                    .expect("the rateable finding list is nonempty");
                lines.push(Line::from(format!(
                    "  Finding {}/{} — [{}] {}",
                    review.cursor.min(findings.len() - 1) + 1,
                    findings.len(),
                    category_label(selected.category),
                    display_atom(&selected.statement, 220)
                )));
                lines.push(Line::from(format!(
                    "  Evidence so far: approve/reject {}/{} | source {}",
                    selected.evidence.explicit_approvals,
                    selected.evidence.explicit_rejections,
                    display_atom(&selected.source_finding, 100)
                )));

                if let Some(experiment) = intelligence
                    .store
                    .experiments
                    .iter()
                    .find(|experiment| experiment.id == experiment_id)
                {
                    let recorded = experiment
                        .finding_outcomes
                        .iter()
                        .filter(|outcome| outcome.knowledge_entry_id == selected.id)
                        .map(|outcome| match outcome.signal {
                            OutcomeSignal::PositiveEndorsement => "liked",
                            OutcomeSignal::RevisionRequested | OutcomeSignal::UserDiscarded => {
                                "needs revision"
                            }
                            _ => "technical/neutral",
                        })
                        .collect::<Vec<_>>();
                    if !recorded.is_empty() {
                        lines.push(Line::from(format!(
                            "  Already recorded for this experiment: {}",
                            recorded.join(", ")
                        )));
                    }
                }
            }
        }
        None => lines.push(Line::from(
            "No active experiment. Generate a composition to rate one of its findings.",
        )),
    }

    lines.push(Line::from(""));
    lines.push(Line::from(
        "Up/Down: choose finding | Y: this finding helped | N: this finding needs revision | Esc/Ctrl+K: close",
    ));
    let block = Paragraph::new(lines).wrap(Wrap { trim: false }).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(" Engram Memory Review & Feedback | Ctrl+K "),
    );
    frame.render_widget(block, area);
}

fn category_label(category: MemoryCategory) -> &'static str {
    match category {
        MemoryCategory::Technical => "technical",
        MemoryCategory::Musical => "musical",
        MemoryCategory::Experiential => "experiential",
    }
}

fn display_atom(value: &str, max_chars: usize) -> String {
    let mut rendered = String::new();
    for ch in value.chars().filter(|ch| !ch.is_control()) {
        if rendered.chars().count() >= max_chars {
            rendered.push('…');
            break;
        }
        rendered.push(ch);
    }
    rendered
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}
