use modalx::prelude::*;

fn main() -> modalx::Result<()> {
    let mut selected_row = 0;

    let columns = vec![
        TableColumn::new("PID", 8).right_aligned(),
        TableColumn::new("Process Name", 24),
        TableColumn::new("Memory (MB)", 14).right_aligned(),
        TableColumn::new("CPU (%)", 10).right_aligned(),
        TableColumn::new("Status", 12),
    ];

    let rows = vec![
        vec![
            "1042".into(),
            "systemd".into(),
            "48.2".into(),
            "0.1".into(),
            "Running".into(),
        ],
        vec![
            "2180".into(),
            "dockerd".into(),
            "312.5".into(),
            "1.4".into(),
            "Running".into(),
        ],
        vec![
            "3401".into(),
            "postgres".into(),
            "524.8".into(),
            "2.1".into(),
            "Running".into(),
        ],
        vec![
            "4892".into(),
            "redis-server".into(),
            "84.1".into(),
            "0.3".into(),
            "Running".into(),
        ],
        vec![
            "5110".into(),
            "nginx".into(),
            "32.0".into(),
            "0.2".into(),
            "Running".into(),
        ],
        vec![
            "6720".into(),
            "craft-daemon".into(),
            "41.6".into(),
            "0.0".into(),
            "Running".into(),
        ],
        vec![
            "7831".into(),
            "node (frontend)".into(),
            "189.4".into(),
            "0.8".into(),
            "Running".into(),
        ],
    ];

    let modal = TableModal::new("SYSTEM PROCESS VIEWER")
        .with_columns(columns)
        .with_rows(rows)
        .with_selectable(true);

    match modal.run(&mut selected_row)? {
        TableOutcome::Selected(idx) => {
            println!("Selected process row index: {}", idx);
        }
        TableOutcome::Cancelled => {
            println!("Table viewer closed.");
        }
    }

    Ok(())
}
