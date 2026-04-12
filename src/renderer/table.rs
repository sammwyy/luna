use crate::renderer::markup;

pub struct Table {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

impl Table {
    pub fn new(headers: Vec<String>) -> Self {
        Self {
            headers,
            rows: Vec::new(),
        }
    }

    pub fn add_row(&mut self, row: Vec<String>) {
        self.rows.push(row);
    }

    pub fn render(&self) -> String {
        if self.headers.is_empty() && self.rows.is_empty() {
            return String::new();
        }

        let num_cols = if !self.headers.is_empty() {
            self.headers.len()
        } else {
            self.rows.first().map(|r| r.len()).unwrap_or(0)
        };

        // Calculate max width for each column (using stripped text length)
        let mut widths = vec![0; num_cols];

        for (i, h) in self.headers.iter().enumerate() {
            let plain = markup::strip_ansi(h);
            widths[i] = widths[i].max(plain.len());
        }

        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if i < num_cols {
                    let plain = markup::strip_ansi(cell);
                    widths[i] = widths[i].max(plain.len());
                }
            }
        }

        let mut out = String::new();

        // Render headers
        if !self.headers.is_empty() {
            for (i, h) in self.headers.iter().enumerate() {
                let plain = markup::strip_ansi(h);
                let padding = widths[i] - plain.len();
                out.push_str(h);
                for _ in 0..padding {
                    out.push(' ');
                }
                if i < num_cols - 1 {
                    out.push_str("  ");
                }
            }
            out.push('\n');

            // Header separator (using theme border color if possible, or just a line)
            for (i, w) in widths.iter().enumerate() {
                out.push_str("<color_border>");
                for _ in 0..*w {
                    out.push('─');
                }
                out.push_str("</color_border>");
                if i < num_cols - 1 {
                    out.push_str("  ");
                }
            }
            out.push('\n');
        }

        // Render rows
        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if i < num_cols {
                    let plain = markup::strip_ansi(cell);
                    let padding = widths[i] - plain.len();
                    out.push_str(cell);
                    for _ in 0..padding {
                        out.push(' ');
                    }
                    if i < num_cols - 1 {
                        out.push_str("  ");
                    }
                }
            }
            out.push('\n');
        }

        out
    }
}
