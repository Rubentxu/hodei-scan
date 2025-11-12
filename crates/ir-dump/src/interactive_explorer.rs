//! Interactive Explorer
//!
//! REPL for exploring IR data interactively

use hodei_ir::FindingSet;
use reedline::{DefaultPrompt, Reedline, Signal};
use std::sync::Arc;

/// Interactive IR explorer
pub struct InteractiveExplorer {
    ir: Arc<FindingSet>,
    current_index: usize,
    reedline: Reedline,
}

impl InteractiveExplorer {
    pub fn new(ir: FindingSet) -> Self {
        Self {
            ir: Arc::new(ir),
            current_index: 0,
            reedline: Reedline::create(),
        }
    }

    /// Start the interactive explorer
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("hodei-scan IR Explorer");
        println!("Type 'help' for available commands");
        println!("Type 'quit' to exit\n");

        loop {
            let prompt = format!("[IR {}]> ", self.current_index + 1);

            match self.reedline.read_line(&DefaultPrompt::default()) {
                Ok(Signal::Success(buffer)) => {
                    let input = buffer.as_str();
                    if !self.handle_command(input).await? {
                        break;
                    }
                }
                Ok(Signal::CtrlD | Signal::CtrlC) => {
                    println!("\nGoodbye!");
                    break;
                }
                Err(err) => {
                    eprintln!("Error reading input: {:?}", err);
                    break;
                }
            }
        }

        Ok(())
    }

    async fn handle_command(&mut self, input: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let parts: Vec<&str> = input.split_whitespace().collect();

        if parts.is_empty() {
            return Ok(true);
        }

        match parts[0] {
            "help" => self.show_help(),
            "show" => self.show_current_finding(),
            "next" => self.next_finding(),
            "prev" => self.prev_finding(),
            "goto" if parts.len() > 1 => {
                if let Ok(idx) = parts[1].parse::<usize>() {
                    self.goto_finding(idx - 1)?;
                }
            }
            "list" => self.list_findings(),
            "filter" if parts.len() > 1 => self.filter_findings(&parts[1..]),
            "stats" => self.show_stats(),
            "quit" | "exit" => return Ok(false),
            _ => println!("Unknown command. Type 'help' for available commands."),
        }

        Ok(true)
    }

    fn show_help(&self) {
        println!("Available commands:");
        println!("  show      - Show current finding");
        println!("  next      - Move to next finding");
        println!("  prev      - Move to previous finding");
        println!("  goto N    - Jump to finding N (1-based)");
        println!("  list      - List all findings");
        println!("  filter T  - Filter by fact type");
        println!("  stats     - Show statistics");
        println!("  help      - Show this help");
        println!("  quit      - Exit explorer");
    }

    fn show_current_finding(&self) {
        if let Some(finding) = self.ir.get(self.current_index) {
            println!("\n{}", "=".repeat(60));
            println!("Finding #{} (of {})", self.current_index + 1, self.ir.len());
            println!("{}", "=".repeat(60));
            println!("Fact Type: {}", finding.fact_type);
            println!("Message: {}", finding.message);
            println!("Location: {}", finding.location);
            println!("{}", "=".repeat(60));
            println!();
        } else {
            println!("No findings to show");
        }
    }

    fn next_finding(&mut self) {
        if self.current_index < self.ir.len().saturating_sub(1) {
            self.current_index += 1;
            self.show_current_finding();
        } else {
            println!("Already at last finding");
        }
    }

    fn prev_finding(&mut self) {
        if self.current_index > 0 {
            self.current_index -= 1;
            self.show_current_finding();
        } else {
            println!("Already at first finding");
        }
    }

    fn goto_finding(&mut self, index: usize) -> Result<(), Box<dyn std::error::Error>> {
        if index < self.ir.len() {
            self.current_index = index;
            self.show_current_finding();
        } else {
            println!("Invalid index. Use 'list' to see all findings");
        }
        Ok(())
    }

    fn list_findings(&self) {
        println!("\nAll findings:");
        for (i, finding) in self.ir.iter().enumerate() {
            let marker = if i == self.current_index { ">" } else { " " };
            println!("{} [{}] {}", marker, i + 1, finding.message);
        }
        println!();
    }

    fn filter_findings(&self, filter_parts: &[&str]) {
        let filter = filter_parts.join(" ");
        println!("\nFiltering by: '{}'", filter);
        println!("{}", "-".repeat(60));

        for (i, finding) in self.ir.iter().enumerate() {
            let fact_type_str = format!("{}", finding.fact_type);
            if fact_type_str
                .to_lowercase()
                .contains(&filter.to_lowercase())
            {
                println!("[{}] {} - {}", i + 1, finding.fact_type, finding.message);
            }
        }
        println!();
    }

    fn show_stats(&self) {
        let mut by_type = std::collections::HashMap::new();
        for finding in self.ir.as_ref() {
            *by_type.entry(&finding.fact_type).or_insert(0) += 1;
        }

        println!("\nStatistics:");
        println!("{}", "=".repeat(60));
        println!("Total findings: {}", self.ir.len());
        println!("\nBy fact type:");
        for (fact_type, count) in by_type {
            println!("  {}: {}", fact_type, count);
        }
        println!("{}", "=".repeat(60));
        println!();
    }
}
