use std::{cmp::Ordering, collections::HashMap};

use crate::data_types::{ConfigInstance, Label, LabelType};

struct LabelHolder {
    label_type: LabelType,
    label: Label,
}

/// labels = selected labels, label_types = all label types avaiable, instances = all instances to select from
pub fn select_instance(
    instances: Vec<ConfigInstance>,
    labels: Vec<Label>,
    label_types: Vec<LabelType>,
) -> Option<ConfigInstance> {
    let label_type_map: HashMap<String, LabelType> = label_types
        .into_iter()
        .map(|label| (label.name.to_owned(), label))
        .collect();
    let selected_label_type_map: HashMap<String, &Label> = labels
        .iter()
        .map(|label| (label.label_type.to_owned(), label.clone()))
        .collect();
    let label_count = labels.len();

    let mut matched_instance: Option<ConfigInstance> = None;
    let mut matched_instance_labels: Vec<&Label> = vec![];

    for instance in instances {
        if instance.labels == labels {
            return Some(instance);
        }

        let mut matched_labels: Vec<&Label> = vec![];

        for label in &instance.labels {
            let label_type = label_type_map.get(&label.label_type).unwrap(); // todo: handle
            let selected_label = selected_label_type_map.get(&label_type.name);
            match selected_label {
                Some(selected_label) => {
                    if selected_label.value == label.value {
                        matched_labels.push(selected_label.to_owned());
                    }
                }
                _ => {
                    continue;
                }
            }
        }

        if label_count > matched_labels.len() {
            // missing label, cannot select
            continue;
        }

        if matched_labels.len() > matched_instance_labels.len() {
            matched_instance = Some(instance);
            matched_instance_labels = matched_labels;
        } else {
            // IF THE MATCHING LABELS ARE THE SAME, CHECK IF THE LABELS ARE HIGHER PRIORITY
            matched_labels.sort_by(|a, b| order_by_priority(&a, &b, &label_type_map));
            matched_instance_labels.sort_by(|a, b| order_by_priority(&a, &b, &label_type_map));

            for i in 1..matched_labels.len() {
                let lbl = label_type_map
                    .get(&matched_labels.get(i).unwrap().label_type)
                    .unwrap(); // todo: handle
                let matched_lbl = label_type_map
                    .get(&matched_instance_labels.get(i).unwrap().label_type)
                    .unwrap(); // todo: handle

                if lbl.priority > matched_lbl.priority {
                    matched_instance = Some(instance);
                    matched_instance_labels = matched_labels;
                    break;
                }
            }
        }
    }

    return matched_instance;
}

fn order_by_priority(
    a: &Label,
    b: &Label,
    label_type_map: &HashMap<String, LabelType>,
) -> Ordering {
    if let Some(a_type) = label_type_map.get(&a.label_type) {
        if let Some(b_type) = label_type_map.get(&b.label_type) {
            return a_type.priority.cmp(&b_type.priority);
        }
        return Ordering::Greater;
    }
    return Ordering::Less;
}

fn select_instance_new(label_types: Vec<LabelType>) {}
