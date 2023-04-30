use std::{cmp::Ordering, collections::HashMap};
use log::{debug, warn};
use yak_man_core::model::{ConfigInstance, Label, LabelType};



/// Common logic to select a config instance from a selected labels
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
        .map(|label| (label.label_type.to_owned(), label))
        .collect();
    let label_count = labels.len();

    let mut matched_instance: Option<ConfigInstance> = None;
    let mut matched_instance_labels: Vec<&Label> = vec![];

    // todo!("Need to fix this logic after moving labels to revisions");

    for instance in instances {
        if instance.labels == labels {
            // All labels are a perfect match, just return early
            return Some(instance);
        }

        // Find all matching labels for this instance
        let mut matched_labels: Vec<&Label> = vec![];
        for label in &instance.labels {

            let label_type = match label_type_map.get(&label.label_type) {
                Some(l) => l,
                None => {
                    warn!("Invalid label found {label:?}, ignoring");
                    continue;
                }
            };

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

        // If the current instance is missing a label, it is not eligible, so continue to the next instance
        if label_count > matched_labels.len() {
            continue;
        }

        if matched_labels.len() > matched_instance_labels.len() {
            matched_instance = Some(instance);
            matched_instance_labels = matched_labels;
        } else {
            // IF THE MATCHING LABELS ARE THE SAME, CHECK IF THE LABELS ARE HIGHER PRIORITY
            matched_labels.sort_by(|a, b| order_by_priority(a, b, &label_type_map));
            matched_instance_labels.sort_by(|a, b| order_by_priority(a, b, &label_type_map));

            for i in 0..matched_labels.len() {
                let lbl = label_type_map
                    .get(&matched_labels.get(i).unwrap().label_type)
                    .unwrap(); // todo: handle
                let matched_lbl = label_type_map
                    .get(&matched_instance_labels.get(i).unwrap().label_type)
                    .unwrap(); // todo: handle

                if lbl.priority > matched_lbl.priority {
                    debug!("Found better match");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_instance_exact_match() { 
        let instances = vec![
            ConfigInstance {
                config_name: "config1".to_owned(),
                instance: "instance1".to_owned(),
                labels: vec![
                    Label {
                        label_type: "environment".to_owned(),
                        value: "dev".to_owned(),
                    },
                    Label {
                        label_type: "service".to_owned(),
                        value: "api".to_owned(),
                    },
                ],
                current_revision: "test".to_string(),
                revisions: vec![],
                pending_revision: None,
                changelog: vec![]
            },
            ConfigInstance {
                config_name: "config1".to_owned(),
                instance: "instance2".to_owned(),
                labels: vec![
                    Label {
                        label_type: "environment".to_owned(),
                        value: "prod".to_owned(),
                    },
                    Label {
                        label_type: "service".to_owned(),
                        value: "api".to_owned(),
                    },
                ],
                current_revision: "test".to_string(),
                revisions: vec![],
                pending_revision: None,
                changelog: vec![]
            },
        ];
        let labels = vec![
            Label {
                label_type: "environment".to_owned(),
                value: "dev".to_owned(),
            },
            Label {
                label_type: "service".to_owned(),
                value: "api".to_owned(),
            },
        ];
        let label_types = vec![
            LabelType {
                name: "environment".to_owned(),
                description: "".to_owned(),
                priority: 1,
                options: vec!["dev".to_owned(), "prod".to_owned()],
            },
            LabelType {
                name: "service".to_owned(),
                description: "".to_owned(),
                priority: 2,
                options: vec!["api".to_owned(), "web".to_owned()],
            },
        ];

        let result = select_instance(instances, labels, label_types);

        assert!(result.is_some());
        let result = result.unwrap();

        assert_eq!("config1", result.config_name);
        assert_eq!("instance1", result.instance);
        assert_eq!(2, result.labels.len());
        assert_eq!("environment", result.labels[0].label_type);
        assert_eq!("dev", result.labels[0].value);
        assert_eq!("service", result.labels[1].label_type);
        assert_eq!("api", result.labels[1].value);
    }

    #[test]
    fn test_select_instance_partial_match() {
        let instances = vec![
            ConfigInstance {
                config_name: "config1".to_owned(),
                instance: "instance1".to_owned(),
                labels: vec![
                    Label {
                        label_type: "environment".to_owned(),
                        value: "dev".to_owned(),
                    },
                    Label {
                        label_type: "service".to_owned(),
                        value: "api".to_owned(),
                    },
                ],
                current_revision: "test".to_string(),
                revisions: vec![],
                pending_revision: None,
                changelog: vec![]
            },
            ConfigInstance {
                config_name: "config1".to_owned(),
                instance: "instance2".to_owned(),
                labels: vec![
                    Label {
                        label_type: "environment".to_owned(),
                        value: "prod".to_owned(),
                    },
                    Label {
                        label_type: "service".to_owned(),
                        value: "api".to_owned(),
                    },
                ],
                current_revision: "test".to_string(),
                revisions: vec![],
                pending_revision: None,
                changelog: vec![]
            },
            ConfigInstance {
                config_name: "config1".to_owned(),
                instance: "instance3".to_owned(),
                labels: vec![
                    Label {
                        label_type: "environment".to_owned(),
                        value: "prod".to_owned(),
                    },
                    Label {
                        label_type: "service".to_owned(),
                        value: "web".to_owned(),
                    },
                ],
                current_revision: "test".to_string(),
                revisions: vec![],
                pending_revision: None,
                changelog: vec![]
            },
        ];
        let labels = vec![Label {
            label_type: "service".to_owned(),
            value: "api".to_owned(),
        }];
        let label_types = vec![
            LabelType {
                name: "environment".to_owned(),
                priority: 1,
                description: "".to_owned(),
                options: vec!["dev".to_owned(), "prod".to_owned()],
            },
            LabelType {
                name: "service".to_owned(),
                priority: 2,
                description: "".to_owned(),
                options: vec!["api".to_owned(), "web".to_owned()],
            },
        ];

        let result = select_instance(instances, labels, label_types);

        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!("config1", result.config_name);
        assert_eq!("instance1", result.instance);
        assert_eq!(2, result.labels.len());
        assert_eq!("environment", result.labels[0].label_type);
        assert_eq!("dev", result.labels[0].value);
        assert_eq!("service", result.labels[1].label_type);
        assert_eq!("api", result.labels[1].value);
    }

    #[test]
    fn test_no_instance_match() {
        let instances = vec![
            ConfigInstance {
                config_name: "instance1_config".to_owned(),
                instance: "instance1".to_owned(),
                labels: vec![
                    Label {
                        label_type: "env".to_owned(),
                        value: "dev".to_owned(),
                    },
                    Label {
                        label_type: "app".to_owned(),
                        value: "frontend".to_owned(),
                    },
                ],
                current_revision: "test".to_string(),
                revisions: vec![],
                pending_revision: None,
                changelog: vec![]
            },
            ConfigInstance {
                config_name: "instance2_config".to_owned(),
                instance: "instance2".to_owned(),
                labels: vec![
                    Label {
                        label_type: "env".to_owned(),
                        value: "prod".to_owned(),
                    },
                    Label {
                        label_type: "app".to_owned(),
                        value: "backend".to_owned(),
                    },
                ],
                current_revision: "test".to_string(),
                revisions: vec![],
                pending_revision: None,
                changelog: vec![]
            },
        ];

        let labels = vec![
            Label {
                label_type: "env".to_owned(),
                value: "staging".to_owned(),
            },
            Label {
                label_type: "app".to_owned(),
                value: "frontend".to_owned(),
            },
        ];

        let label_types = vec![
            LabelType {
                name: "env".to_owned(),
                priority: 1,
                description: "".to_owned(),
                options: vec![],
            },
            LabelType {
                name: "app".to_owned(),
                priority: 2,
                description: "".to_owned(),
                options: vec![],
            },
        ];

        let selected_instance = select_instance(instances, labels, label_types);
        assert!(selected_instance.is_none());
    }
}
