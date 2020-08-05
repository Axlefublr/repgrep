use std::collections::VecDeque;
use std::io::{self, BufRead, BufReader, Read, Write};

use anyhow::{anyhow, Result};

use crate::rg::de::RgMessage;

pub fn read_messages<R: Read>(rdr: R) -> Result<VecDeque<RgMessage>> {
    let mut saw_match_message = false;

    let mut rg_messages: VecDeque<RgMessage> = VecDeque::new();
    let reader = BufReader::new(rdr);
    for (i, line) in reader.lines().enumerate() {
        // For large result lists show some progress in the terminal.
        if i > 0 && i % 1000 == 0 {
            let _ = io::stdout().write_all(format!("\rMatches found: ~{}", i).as_bytes());
            let _ = io::stdout().flush();
        }

        let rg_msg: RgMessage =
            serde_json::from_str(&line?).map_err(|e| anyhow!("Failed to parse JSON: {}", e))?;

        if !saw_match_message && matches!(rg_msg, RgMessage::Match { .. }) {
            saw_match_message = true;
        }

        rg_messages.push_back(rg_msg);
    }

    // We expect at least one message.
    if !saw_match_message {
        Err(anyhow!("No matches returned from rg!"))
    } else {
        Ok(rg_messages)
    }
}
