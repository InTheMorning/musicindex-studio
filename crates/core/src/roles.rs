//! The role list of a credit (ADR 0112).
//!
//! The core owns the list. The API gives it to the UI. The draft save uses it
//! to write the group of a role (see `apply_role_group` in the pipeline).
//!
//! Source of the names: the group "Music" is the group of MSP 2.0. The other
//! groups and roles are from `taxonomy.json` of the Podcast Taxonomy Project
//! (podcast-namespace repository, commit
//! `f593c5da3f6ef680cd80b92d96078795744dac0b`, license CC0 1.0). This file
//! copies only the names of the groups and the roles.

/// A group of roles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoleGroup {
    /// The display name of the group.
    pub name: &'static str,
    /// The display names of the roles, in sequence.
    pub roles: &'static [&'static str],
}

const GROUPS: &[RoleGroup] = &[
    RoleGroup {
        name: "Music",
        roles: &[
            "Band",
            "Vocalist",
            "Guitarist",
            "Bassist",
            "Drummer",
            "Keyboardist",
            "Musician",
        ],
    },
    RoleGroup {
        name: "Creative Direction",
        roles: &[
            "Director",
            "Assistant Director",
            "Executive Producer",
            "Senior Producer",
            "Producer",
            "Associate Producer",
            "Development Producer",
            "Creative Director",
        ],
    },
    RoleGroup {
        name: "Cast",
        roles: &[
            "Host",
            "Co-Host",
            "Guest Host",
            "Guest",
            "Voice Actor",
            "Narrator",
            "Announcer",
            "Reporter",
        ],
    },
    RoleGroup {
        name: "Writing",
        roles: &[
            "Author",
            "Editorial Director",
            "Co-Writer",
            "Writer",
            "Songwriter",
            "Guest Writer",
            "Story Editor",
            "Managing Editor",
            "Script Editor",
            "Script Coordinator",
            "Researcher",
            "Editor",
            "Fact Checker",
            "Translator",
            "Transcriber",
            "Logger",
        ],
    },
    RoleGroup {
        name: "Audio Production",
        roles: &[
            "Studio Coordinator",
            "Technical Director",
            "Technical Manager",
            "Audio Engineer",
            "Remote Recording Engineer",
            "Post Production Engineer",
        ],
    },
    RoleGroup {
        name: "Audio Post-Production",
        roles: &[
            "Audio Editor",
            "Sound Designer",
            "Foley Artist",
            "Composer",
            "Theme Music",
            "Music Production",
            "Music Contributor",
        ],
    },
    RoleGroup {
        name: "Administration",
        roles: &[
            "Production Coordinator",
            "Booking Coordinator",
            "Production Assistant",
            "Content Manager",
            "Marketing Manager",
            "Sales Representative",
            "Sales Manager",
        ],
    },
    RoleGroup {
        name: "Visuals",
        roles: &["Graphic Designer", "Cover Art Designer"],
    },
    RoleGroup {
        name: "Community",
        roles: &["Social Media Manager"],
    },
    RoleGroup {
        name: "Misc.",
        roles: &["Consultant", "Intern"],
    },
    RoleGroup {
        name: "Video Production",
        roles: &[
            "Camera Operator",
            "Lighting Designer",
            "Camera Grip",
            "Assistant Camera",
        ],
    },
    RoleGroup {
        name: "Video Post-Production",
        roles: &["Editor", "Assistant Editor"],
    },
];

/// The groups in sequence: "Music" first, then the groups of the taxonomy.
#[must_use]
pub fn groups() -> &'static [RoleGroup] {
    GROUPS
}

fn same(a: &str, b: &str) -> bool {
    a.trim().eq_ignore_ascii_case(b.trim())
}

/// The display names of the groups that hold `role`, in the sequence of the
/// list. The comparison ignores case and the space at each end. A role that
/// is not of the list gives an empty list.
#[must_use]
pub fn groups_of(role: &str) -> Vec<&'static str> {
    GROUPS
        .iter()
        .filter(|g| g.roles.iter().any(|r| same(r, role)))
        .map(|g| g.name)
        .collect()
}

/// The form that Studio writes in a feed: lowercase text with no space at
/// each end (ADR 0112 item 5).
#[must_use]
pub fn wire(text: &str) -> String {
    text.trim().to_lowercase()
}

/// Whether `group` is `name`, with no regard to case and to the space at
/// each end.
pub(crate) fn is_group(group: &str, name: &str) -> bool {
    same(group, name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_list_has_the_music_group_and_each_role_of_the_taxonomy() {
        let all = groups();
        assert_eq!(
            all[0].name, "Music",
            "ADR 0112 item 1: the group Music must be first in the list. Move it to the start of GROUPS."
        );
        assert_eq!(
            all.len(),
            12,
            "ADR 0112 item 1: the list must have Music and the 11 groups of taxonomy.json. Compare GROUPS with the file at the commit that roles.rs names."
        );
        let taxonomy: usize = all[1..].iter().map(|g| g.roles.len()).sum();
        assert_eq!(
            taxonomy, 63,
            "ADR 0112 context: taxonomy.json has 63 roles. Compare GROUPS with the file at the commit that roles.rs names."
        );
        assert_eq!(all[0].roles.len(), 7);
        for role in ["Songwriter", "Composer", "Cover Art Designer", "Vocalist"] {
            assert!(
                !groups_of(role).is_empty(),
                "ADR 0112 item 1: the role {role} must be in the list. Add it to GROUPS."
            );
        }
    }

    #[test]
    fn editor_is_in_two_groups() {
        assert_eq!(groups_of("Editor"), ["Writing", "Video Post-Production"]);
        assert_eq!(groups_of("  editor "), ["Writing", "Video Post-Production"]);
    }

    #[test]
    fn a_role_is_matched_with_no_regard_to_case_and_an_unknown_role_has_no_group() {
        assert_eq!(groups_of("vOcAlIsT"), ["Music"]);
        assert_eq!(groups_of("Field Recordist").len(), 0);
        assert_eq!(groups_of("").len(), 0);
    }

    #[test]
    fn wire_gives_trimmed_lowercase() {
        assert_eq!(wire(" Audio Production "), "audio production");
    }
}
