pub(crate) fn render_table(headers: &[&str], rows: &[Vec<String>]) -> String {
    let mut widths = headers
        .iter()
        .map(|header| display_width(header))
        .collect::<Vec<_>>();

    for row in rows {
        for (index, cell) in row.iter().enumerate().take(widths.len()) {
            widths[index] = widths[index].max(display_width(cell));
        }
    }

    let mut lines = vec![
        render_row(
            &headers
                .iter()
                .map(|header| header.to_string())
                .collect::<Vec<_>>(),
            &widths,
        ),
        widths
            .iter()
            .map(|width| "-".repeat(*width))
            .collect::<Vec<_>>()
            .join("  "),
    ];

    if rows.is_empty() {
        lines.push("No results.".to_owned());
    } else {
        for row in rows {
            lines.push(render_row(row, &widths));
        }
    }

    format!("{}\n", lines.join("\n"))
}

fn render_row(row: &[String], widths: &[usize]) -> String {
    widths
        .iter()
        .enumerate()
        .map(|(index, width)| {
            let cell = row.get(index).map_or("", String::as_str);
            format!(
                "{cell}{}",
                " ".repeat(width.saturating_sub(display_width(cell)))
            )
        })
        .collect::<Vec<_>>()
        .join("  ")
}

fn display_width(value: &str) -> usize {
    value.chars().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_simple_table() {
        let output = render_table(
            &["id", "city"],
            &[
                vec!["INSTALLATION_ID_1".to_owned(), "CITY_1".to_owned()],
                vec!["INSTALLATION_ID_2".to_owned(), "CITY_2".to_owned()],
            ],
        );

        assert!(output.contains("id                 city"));
        assert!(output.contains("INSTALLATION_ID_1  CITY_1"));
        assert!(output.contains("INSTALLATION_ID_2  CITY_2"));
    }

    #[test]
    fn renders_no_results_message() {
        let output = render_table(&["id"], &[]);

        assert!(output.contains("id"));
        assert!(output.contains("No results."));
    }
}
