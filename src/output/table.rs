use comfy_table::{Attribute, Cell, CellAlignment, ContentArrangement, Table, presets::UTF8_FULL};

pub fn standard_table(headers: &[&str]) -> Table {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(
        headers
            .iter()
            .map(|h| Cell::new(*h).add_attribute(Attribute::Bold))
            .collect::<Vec<_>>(),
    );
    table
}

pub fn right(text: impl Into<String>) -> Cell {
    Cell::new(text.into()).set_alignment(CellAlignment::Right)
}

pub fn left(text: impl Into<String>) -> Cell {
    Cell::new(text.into()).set_alignment(CellAlignment::Left)
}

pub fn truncate(input: &str, max_chars: usize) -> String {
    if input.chars().count() <= max_chars {
        return input.to_string();
    }
    if max_chars <= 1 {
        return "…".to_string();
    }
    let mut out = input.chars().take(max_chars - 1).collect::<String>();
    out.push('…');
    out
}
