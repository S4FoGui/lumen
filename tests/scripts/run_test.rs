use std::collections::HashMap;

fn main() {
    let devices = vec![
        "hw:CARD=Device,DEV=0",
        "plughw:CARD=Device,DEV=0",
        "default:CARD=Device",
        "sysdefault:CARD=Device",
        "front:CARD=Device,DEV=0",
        "dsnoop:CARD=Device,DEV=0",
        "hw:CARD=1,DEV=0",
        "plughw:CARD=1,DEV=0",
    ];

    let mut alsa_names = HashMap::new();
    alsa_names.insert("Device".to_string(), "USB PnP Sound Device".to_string());
    alsa_names.insert("1".to_string(), "Camera".to_string());

    let mut categorized_alsa: HashMap<String, (String, String, i32)> = HashMap::new();
    let mut final_list: Vec<(String, String)> = Vec::new();

    for d in devices {
        let id = d.to_string();
        
        if id.starts_with("dmix") || id.starts_with("dsnoop") || id == "sysdefault" {
            continue;
        }

        let mut is_alsa = false;
        let mut current_card_code = String::new();

        if id.starts_with("hw:CARD=") || id.starts_with("plughw:CARD=") || id.starts_with("default:CARD=") || id.starts_with("sysdefault:CARD=") || id.starts_with("front:CARD=") || id.starts_with("dsnoop:CARD=") {
            is_alsa = true;
            let parts: Vec<&str> = id.split(',').collect();
            if let Some(card_part) = parts.first() {
                current_card_code = card_part.split('=').nth(1).unwrap_or("").to_string();
            }
        }

        if is_alsa {
            let score = if id.starts_with("default:CARD=") { 4 }
                       else if id.starts_with("plughw:CARD=") { 3 }
                       else if id.starts_with("sysdefault:CARD=") { 2 }
                       else if id.starts_with("hw:CARD=") { 1 }
                       else { 0 };

            let mut label = id.clone();
            if let Some(pretty) = alsa_names.get(&current_card_code) {
                let prefix = id.split(':').next().unwrap_or("");
                label = format!("{} ({})", pretty, prefix);
            }

            if let Some((_, _, best_score)) = categorized_alsa.get(&current_card_code) {
                if score > *best_score {
                    categorized_alsa.insert(current_card_code.clone(), (id.clone(), label, score));
                }
            } else {
                categorized_alsa.insert(current_card_code.clone(), (id.clone(), label, score));
            }
        } else {
            final_list.push((id.clone(), id.clone()));
        }
    }

    for (_, (id, label, _)) in categorized_alsa {
        final_list.push((id, label));
    }

    final_list.sort_by(|a, b| a.1.cmp(&b.1));

    for item in final_list {
        println!("{} -> {}", item.0, item.1);
    }
}
